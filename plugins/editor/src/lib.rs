use nih_plug::prelude::*;
use passthru::spe_editor_passthru;
use record::{Record, cartestian_changes_msg};
use shared::PASSTHRU_LAYOUT;
use std::sync::Arc;
use xpans_spe_nih::spe::{AxisCombo, Message};
use xpans_spe_nih::{SpeBundle, msg_to_event};
mod params;
use crate::params::*;

mod passthru;
mod record;

#[derive(Default)]
struct SceneEditor {
    params: Arc<PluginParams>,
    was_stopped: bool,
    record: Record,
}

impl SceneEditor {
    fn just_began_playing(&self, context: &mut impl ProcessContext<Self>) -> bool {
        self.was_stopped & context.transport().playing
    }
    fn all_property_msgs(&self) -> [Message<f32>; 2] {
        [self.all_pos(), self.all_ext()]
    }
    fn all_pos(&self) -> Message<f32> {
        Message::position(&AxisCombo::XYZ, self.params.pos())
    }
    fn all_ext(&self) -> Message<f32> {
        Message::extent(&AxisCombo::XYZ, self.params.ext())
    }
    fn changed_pos_msg(&self) -> Option<Message<f32>> {
        cartestian_changes_msg(self.record.pos, self.params.pos(), Message::position)
    }
    fn changed_ext_msg(&self) -> Option<Message<f32>> {
        cartestian_changes_msg(self.record.ext, self.params.ext(), Message::extent)
    }
}

impl Plugin for SceneEditor {
    const NAME: &'static str = "Scene Editor";

    const VENDOR: &'static str = "xpans";

    const URL: &'static str = "xpans.org";

    const EMAIL: &'static str = "contact@xpans.org";

    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[PASSTHRU_LAYOUT];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = SpeBundle;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.just_began_playing(context) {
            for msg in self.all_property_msgs().iter() {
                let event = msg_to_event(0, self.params.id(), *msg);
                context.send_event(event);
            }
        }
        spe_editor_passthru(context, &[self.params.id()]);
        if let Some(msg) = self.changed_pos_msg() {
            let event = msg_to_event(0, self.params.id(), msg);
            context.send_event(event);
        }
        if let Some(msg) = self.changed_ext_msg() {
            let event = msg_to_event(0, self.params.id(), msg);
            context.send_event(event);
        }
        self.record = Record::from(self.params.as_ref());
        self.was_stopped = !context.transport().playing;
        ProcessStatus::Normal
    }
}

#[cfg(feature = "clap")]
impl ClapPlugin for SceneEditor {
    const CLAP_ID: &'static str = "org.xpans.SceneEditor";

    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Generates and edits spatial audio sources");

    const CLAP_MANUAL_URL: Option<&'static str> = None;

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Utility];
}
#[cfg(feature = "clap")]
nih_export_clap!(SceneEditor);

#[cfg(feature = "vst3")]
impl Vst3Plugin for SceneEditor {
    const VST3_CLASS_ID: [u8; 16] = *b"__xpans_Editor__";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Spatial, Vst3SubCategory::Tools];
}
#[cfg(feature = "vst3")]
nih_export_vst3!(SceneEditor);
