use std::{
    collections::BTreeMap,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicI64, Ordering},
    },
};
mod editor;

use crossbeam_channel::{Receiver, Sender};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use shared::{PASSTHRU_LAYOUT, sysex};
use xpans_spe_nih::SpeBundle;
use xpans_spe_nih::spe::{ApplyMessage, Message};
use xpans_xsr::{Changes, Event, Record, Sample};

use crate::editor::{FromEditorMessage, ToEditorMessage};

type RecordMap = BTreeMap<i64, BTreeMap<u16, Changes<f32>>>;
#[derive(Default)]
pub struct SceneExporter {
    recv_from_ui: Option<Receiver<FromEditorMessage>>,
    send_to_ui: Option<Sender<ToEditorMessage>>,
    params: Arc<PluginParams>,
    sample_rate: u64,
    previous_sample_pos: i64,
    current: BTreeMap<u16, Changes<f32>>,
    record_map: RecordMap,
    was_playing: bool,
    exporting: bool,
    was_in_export_range: bool,
    export_path: PathBuf,
    scene_start: i64,
    scene_end: i64,
}
impl SceneExporter {
    fn add_current_event(&mut self, source: u16, event: Message<f32>) {
        let source = self.current.entry(source).or_default();
        source.apply_message(event);
    }
    fn record_event(&mut self, sample: i64, source: u16, event: Message<f32>) {
        let sample = self.record_map.entry(sample).or_default();
        let source = sample.entry(source).or_default();
        source.apply_message(event);
    }
    fn unarm_export(&mut self) {
        if let Some(sender) = &self.send_to_ui {
            let _ = sender.send(ToEditorMessage::ExportUnarm);
        }
        self.exporting = false;
    }
    fn set_scene_start(&mut self, sample: i64) {
        self.params.scene_start.store(sample, Ordering::Release);
        self.scene_start = sample;
    }
    fn set_scene_end(&mut self, sample: i64) {
        self.params.scene_end.store(sample, Ordering::Release);
        self.scene_end = sample;
    }
    fn set_export_path(&mut self, path: PathBuf) {
        let mut locked = self.params.export_path.lock().unwrap();
        *locked = path.clone();
        self.export_path = path.clone();
        if let Some(sender) = &self.send_to_ui {
            let _ = sender.send(ToEditorMessage::SetExportPath(path));
        }
    }
}

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[persist = "export-path"]
    export_path: Mutex<PathBuf>,
    #[persist = "scene-start"]
    scene_start: AtomicI64,
    #[persist = "scene-end"]
    scene_end: AtomicI64,
}
impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            export_path: Mutex::new(Default::default()),
            scene_start: Default::default(),
            scene_end: Default::default(),
        }
    }
}

impl Plugin for SceneExporter {
    const NAME: &'static str = "Scene Exporter";

    const VENDOR: &'static str = "xpans";

    const URL: &'static str = "xpans.org";

    const EMAIL: &'static str = "contact@xpans.org";

    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[PASSTHRU_LAYOUT];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = SpeBundle;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let (send_from_ui, recv_from_ui) = crossbeam_channel::unbounded();
        let (send_to_ui, recv_to_ui) = crossbeam_channel::unbounded();
        self.recv_from_ui = Some(recv_from_ui);
        self.send_to_ui = Some(send_to_ui);
        editor::create(
            self.params.clone(),
            send_from_ui,
            recv_to_ui,
            self.exporting,
            self.params.editor_state.clone(),
        )
    }
    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.scene_start = self.params.scene_start.load(Ordering::Acquire);
        self.scene_end = self.params.scene_end.load(Ordering::Acquire);
        true
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.sample_rate = context.transport().sample_rate.round() as u64;
        let current_sample = context.transport().pos_samples().unwrap_or_default();
        let is_playing = context.transport().playing;

        let just_started_playing = (!self.was_playing) && is_playing;

        if just_started_playing {
            self.current.clear();
            self.record_map.clear();
        }

        while let Ok(msg) = self.recv_from_ui.as_ref().unwrap().try_recv() {
            match msg {
                FromEditorMessage::SetSceneStart => {
                    self.set_scene_start(current_sample);
                }
                FromEditorMessage::SetSceneEnd => {
                    self.set_scene_end(current_sample);
                }
                FromEditorMessage::ExportArm => self.exporting = true,
                FromEditorMessage::ExportUnarm => self.exporting = false,
                FromEditorMessage::SetExportPath(path) => {
                    self.set_export_path(path);
                }
            }
        }

        let starting_sample = self.scene_start;
        let ending_sample = self.scene_end;

        let scene_range = starting_sample..=ending_sample;
        let in_export_range = scene_range.contains(&current_sample);

        let just_entered_export_range = (!self.was_in_export_range) && in_export_range;

        if just_entered_export_range {
            let sample = self.record_map.entry(starting_sample).or_default();
            for (id, current_source) in self.current.iter() {
                let record_source = sample.entry(*id).or_default();
                *record_source = *current_source;
            }
        }

        for (event_sample_offset, bundle) in sysex(context) {
            let event_sample_offset = i64::from(event_sample_offset);
            let event_sample = current_sample + event_sample_offset;
            self.add_current_event(bundle.id, bundle.msg);

            if self.exporting && in_export_range {
                self.record_event(event_sample, bundle.id, bundle.msg);
            }
        }

        if self.exporting && (current_sample >= ending_sample) {
            self.unarm_export();
            write_file(&self.export_path, &mut self.record_map, self.sample_rate);
        }

        self.was_in_export_range = in_export_range;
        self.was_playing = is_playing;
        self.previous_sample_pos = current_sample;
        ProcessStatus::Normal
    }
}

fn write_file<P: AsRef<Path>>(path: P, record: &mut RecordMap, sample_rate: u64) {
    if let Ok(file) = File::create(path) {
        let mut writer = BufWriter::new(file);
        let mut event_buf: Vec<Event<u16, f32>> = vec![];
        let mut sample_buf: Vec<Sample<i64, u16, f32>> = vec![];

        for (sample, sample_events) in record.iter() {
            for (id, event) in sample_events.iter() {
                let event = Event {
                    id: *id,
                    changes: *event,
                };
                event_buf.push(event);
            }
            let xsr_sample = Sample {
                sample: *sample,
                events: event_buf.clone().into_boxed_slice(),
            };
            event_buf.clear();
            sample_buf.push(xsr_sample);
        }

        let recordfile = Record::new(sample_rate, sample_buf.into_boxed_slice());

        let _ = rmp_serde::encode::write(&mut writer, &recordfile);

        record.clear();
    }
}

#[cfg(feature = "clap")]
impl ClapPlugin for SceneExporter {
    const CLAP_ID: &'static str = "org.xpans.SceneExporter";

    const CLAP_DESCRIPTION: Option<&'static str> = None;

    const CLAP_MANUAL_URL: Option<&'static str> = None;

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Utility];
}
#[cfg(feature = "clap")]
nih_export_clap!(SceneExporter);

#[cfg(feature = "vst3")]
impl Vst3Plugin for SceneExporter {
    const VST3_CLASS_ID: [u8; 16] = *b"_Scene_Exporter_";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Mastering,
        Vst3SubCategory::Spatial,
        Vst3SubCategory::Tools,
    ];
}
#[cfg(feature = "vst3")]
nih_export_vst3!(SceneExporter);
