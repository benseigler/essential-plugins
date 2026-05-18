use std::{num::NonZero, sync::Arc};

use nih_plug::prelude::*;

#[derive(Default)]
struct MonoMonitor {}

#[derive(Params)]
struct PluginParams {}

const ALL_IN: NonZero<u32> = unsafe { NonZero::new_unchecked(128) };
const STEREO_OUT: NonZero<u32> = unsafe { NonZero::new_unchecked(128) };

const LAYOUT: AudioIOLayout = AudioIOLayout {
    main_input_channels: Some(ALL_IN),
    main_output_channels: Some(STEREO_OUT),
    aux_input_ports: &[],
    aux_output_ports: &[],
    names: PortNames::const_default(),
};

impl Plugin for MonoMonitor {
    const NAME: &'static str = "Mono Monitor";

    const VENDOR: &'static str = "xpans";

    const URL: &'static str = "xpans.org";

    const EMAIL: &'static str = "contact@xpans.org";

    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[LAYOUT];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        Arc::new(PluginParams {})
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel in 1..buffer.channels() {
            for sample in 0..buffer.samples() {
                buffer.as_slice()[0][sample] += buffer.as_slice()[channel][sample];
                buffer.as_slice()[channel][sample] = 0.;
            }
        }
        for i in 0..buffer.samples() {
            buffer.as_slice()[1][i] = buffer.as_slice_immutable()[0][i];
        }
        ProcessStatus::Normal
    }
}

#[cfg(feature = "clap")]
impl ClapPlugin for MonoMonitor {
    const CLAP_ID: &'static str = "audio.xpans.MonoMonitor";

    const CLAP_DESCRIPTION: Option<&'static str> = None;

    const CLAP_MANUAL_URL: Option<&'static str> = None;

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[];
}

#[cfg(feature = "vst3")]
impl Vst3Plugin for MonoMonitor {
    const VST3_CLASS_ID: [u8; 16] = *b"__Mono_Monitor__";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Spatial,
        Vst3SubCategory::Mono,
        Vst3SubCategory::Tools,
    ];
}

#[cfg(feature = "clap")]
nih_export_clap!(MonoMonitor);

#[cfg(feature = "vst3")]
nih_export_vst3!(MonoMonitor);
