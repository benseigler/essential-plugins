use std::{num::NonZero, sync::Arc};

use nih_plug::prelude::*;
use render::{PluginInput, PluginInputViewer, PluginOutput, PluginSources, RenderHandler};

use shared::PanLawOption;
use xpans_spe_nih::SpeBundle;
use xpans_stereo::{Directional, Positional, Processor, pan_law::PanLaw};
use xpans_violet::{RendererBuilder, SourceInterpreter};

type StereoInterpreter<T> = Box<dyn SourceInterpreter<T, Interpretation = T> + Send>;
type RenderHandlerType = RenderHandler<
    StereoInterpreter<f32>,
    Processor<f32, Box<dyn PanLaw<f32> + Send>>,
    PluginInputViewer<f32>,
>;

struct StereoMonitor {
    params: Arc<PluginParams>,
    render_handler: Option<RenderHandlerType>,
    previous_pan_law: PanLawOption,
    previous_stereo_mode: StereoModeOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum)]
enum StereoModeOption {
    Directional,
    Positional,
}
impl Default for StereoModeOption {
    fn default() -> Self {
        Self::Directional
    }
}

impl StereoModeOption {
    fn get_dyn(&self) -> StereoInterpreter<f32> {
        match self {
            Self::Directional => Box::new(Directional::default()),
            Self::Positional => Box::new(Positional::default()),
        }
    }
}

impl StereoMonitor {
    fn handle_pan_law_change(&mut self) {
        let new_pan_law = self.params.pan_law.value();
        if new_pan_law != self.previous_pan_law {
            let handler = self.render_handler.as_mut().unwrap();
            let processor_mut: &mut Processor<_, _> = handler.renderer.sample_processor_mut();
            processor_mut.set_pan_law(new_pan_law.get_dyn());
        }
        self.previous_pan_law = new_pan_law;
    }
    fn handle_stereo_mode_change(&mut self) {
        let new_stereo_mode = self.params.stereo_mode.value();
        if new_stereo_mode != self.previous_stereo_mode {
            let handler = self.render_handler.as_mut().unwrap();
            let interpreter_mut = handler.renderer.interpreter_mut();
            *interpreter_mut = new_stereo_mode.get_dyn();
        }
        self.previous_stereo_mode = new_stereo_mode;
    }
}

impl Default for StereoMonitor {
    fn default() -> Self {
        Self {
            params: Default::default(),
            render_handler: None,
            previous_pan_law: Default::default(),
            previous_stereo_mode: Default::default(),
        }
    }
}

#[derive(Params)]
struct PluginParams {
    #[id = "Pan Law"]
    pan_law: EnumParam<PanLawOption>,
    #[id = "Stereo Mode"]
    stereo_mode: EnumParam<StereoModeOption>,
}
impl Default for PluginParams {
    fn default() -> Self {
        Self {
            pan_law: EnumParam::new("Pan Law", PanLawOption::Sine),
            stereo_mode: EnumParam::new("Stereo Mode", StereoModeOption::Directional),
        }
    }
}

const ALL_IN: NonZero<u32> = unsafe { NonZero::new_unchecked(128) };
const ALL_OUT: NonZero<u32> = unsafe { NonZero::new_unchecked(128) };

const LAYOUT: AudioIOLayout = AudioIOLayout {
    main_input_channels: Some(ALL_IN),
    main_output_channels: Some(ALL_OUT),
    aux_input_ports: &[],
    aux_output_ports: &[],
    names: PortNames::const_default(),
};

impl Plugin for StereoMonitor {
    const NAME: &'static str = "Stereo Monitor";

    const VENDOR: &'static str = "xpans";

    const URL: &'static str = "xpans.audio";

    const EMAIL: &'static str = "contact@xpans.audio";

    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[LAYOUT];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = SpeBundle;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate.round() as u32;
        let buffer_length = buffer_config.max_buffer_size as usize;
        let (audio_mutator, audio_viewer) = PluginInput::new(1, buffer_length, sample_rate, 128);
        let (sources_mutator, sources_viewer) = PluginSources::new(128, buffer_length);

        let stereo_mode_option = self.params.stereo_mode.value();
        let stereo_mode_dyn: StereoInterpreter<f32> = stereo_mode_option.get_dyn();
        self.previous_stereo_mode = stereo_mode_option;

        let pan_law_option = self.params.pan_law.value();
        let pan_law_dyn: Box<dyn PanLaw<f32> + Send> = pan_law_option.get_dyn();
        self.previous_pan_law = pan_law_option;

        let builder = RendererBuilder::new();
        let renderer = builder
            .set_source_interpreter(stereo_mode_dyn)
            .set_sample_processor(Processor::new(pan_law_dyn))
            .set_audio_input(audio_viewer)
            .set_spatial_input(sources_viewer)
            .set_audio_output(PluginOutput::new(2, buffer_length))
            .build()
            .unwrap();
        let handler = RenderHandler::new(renderer, audio_mutator, sources_mutator);
        self.render_handler = Some(handler);

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.handle_stereo_mode_change();
        self.handle_pan_law_change();
        self.render_handler
            .as_mut()
            .unwrap()
            .render(buffer, context);
        ProcessStatus::Normal
    }
}

#[cfg(feature = "clap")]
impl ClapPlugin for StereoMonitor {
    const CLAP_ID: &'static str = "audio.xpans.StereoMonitor";

    const CLAP_DESCRIPTION: Option<&'static str> = None;

    const CLAP_MANUAL_URL: Option<&'static str> = None;

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[];
}

#[cfg(feature = "vst3")]
impl Vst3Plugin for StereoMonitor {
    const VST3_CLASS_ID: [u8; 16] = *b"_Stereo_Monitor_";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Spatial,
        Vst3SubCategory::Stereo,
        Vst3SubCategory::Tools,
    ];
}

#[cfg(feature = "clap")]
nih_export_clap!(StereoMonitor);

#[cfg(feature = "vst3")]
nih_export_vst3!(StereoMonitor);
