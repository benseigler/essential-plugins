use std::{num::NonZero, sync::Arc};

use nih_plug::prelude::*;
use render::{PluginInput, PluginInputViewer, PluginOutput, PluginSources, RenderHandler};
use shared::PanLawOption;
use xpans_headphones::distance::{DistanceCurve, Exponential, Linear};
use xpans_headphones::{Interpreter, Processor, pan_law::PanLaw};
use xpans_spe_nih::SpeBundle;
use xpans_violet::{RendererBuilder, audio_input::interpolation::linear::LinearInterpolator};

type RenderHandlerType = RenderHandler<
    Interpreter<f32>,
    Processor<f32, Box<dyn PanLaw<f32> + Send>, Box<dyn DistanceCurve<f32> + Send>>,
    LinearInterpolator<PluginInputViewer<f32>>,
>;

#[derive(Default)]
struct HeadphoneMonitor {
    params: Arc<PluginParams>,
    render_handler: Option<RenderHandlerType>,
    previous_distance_curve: DistanceCurveOption,
    previous_pan_law: PanLawOption,
}

impl HeadphoneMonitor {
    fn handle_distance_curve_change(&mut self) {
        let new_curve = self.params.distance_curve.value();
        if new_curve != self.previous_distance_curve {
            let handler = self.render_handler.as_mut().unwrap();
            let processor_mut: &mut Processor<_, _, _> = handler.renderer.sample_processor_mut();
            processor_mut.set_distance_curve(new_curve.get_dyn());
        }
        self.previous_distance_curve = new_curve;
    }
    fn handle_pan_law_change(&mut self) {
        let new_pan_law = self.params.pan_law.value();
        if new_pan_law != self.previous_pan_law {
            let handler = self.render_handler.as_mut().unwrap();
            let processor_mut: &mut Processor<_, _, _> = handler.renderer.sample_processor_mut();
            processor_mut.set_pan_law(new_pan_law.get_dyn());
        }
        self.previous_pan_law = new_pan_law;
    }
    fn itd_nanos(&self) -> u32 {
        let itd_millis = self.params.max_itd.value();
        (itd_millis * 1_000_000_f32) as u32
    }
}

#[derive(Enum, PartialEq, Eq)]
enum DistanceCurveOption {
    #[name = "Linear"]
    Linear,
    #[name = "Exponential"]
    Exponential,
}
impl Default for DistanceCurveOption {
    fn default() -> Self {
        Self::Exponential
    }
}

impl DistanceCurveOption {
    fn get_dyn(&self) -> Box<dyn DistanceCurve<f32> + Send> {
        match self {
            DistanceCurveOption::Linear => Box::new(Linear),
            DistanceCurveOption::Exponential => Box::new(Exponential),
        }
    }
}

#[derive(Params)]
pub struct PluginParams {
    #[id = "Pan Law"]
    pan_law: EnumParam<PanLawOption>,
    #[id = "Maximum ITD"]
    max_itd: FloatParam,
    #[id = "Distance Effect"]
    distance_effect: FloatParam,
    #[id = "Distance Curve"]
    distance_curve: EnumParam<DistanceCurveOption>,
    #[id = "Min Distance"]
    min_distance: FloatParam,
    #[id = "Max Distance"]
    max_distance: FloatParam,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            pan_law: EnumParam::new("Pan Law", PanLawOption::Sine),
            max_itd: FloatParam::new(
                "Maximum ITD",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" ms"),
            distance_effect: FloatParam::new(
                "Distance Effect",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            distance_curve: EnumParam::new("Distance Curve", DistanceCurveOption::Exponential),
            min_distance: FloatParam::new(
                "Min Distance",
                0.1,
                FloatRange::Linear { min: 0.0, max: 2. },
            ),
            max_distance: FloatParam::new(
                "Max Distance",
                1.,
                FloatRange::Linear { min: 0.0, max: 2. },
            ),
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

impl Plugin for HeadphoneMonitor {
    const NAME: &'static str = "Headphone Monitor";

    const VENDOR: &'static str = "xpans";

    const URL: &'static str = "xpans.org";

    const EMAIL: &'static str = "contact@xpans.org";

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

        let distance_curve_option = self.params.distance_curve.value();
        let distance_curve_dyn: Box<dyn DistanceCurve<f32> + Send> =
            distance_curve_option.get_dyn();
        self.previous_distance_curve = distance_curve_option;

        let pan_law_option = self.params.pan_law.value();
        let pan_law_dyn: Box<dyn PanLaw<f32> + Send> = pan_law_option.get_dyn();
        self.previous_pan_law = pan_law_option;

        let processor = Processor::new(
            pan_law_dyn,
            self.itd_nanos(),
            distance_curve_dyn,
            self.params.distance_effect.value(),
            self.params.min_distance.value(),
            self.params.max_distance.value(),
        );
        let delay_len = (sample_rate as usize / 1000) + 1;
        let (audio_mutator, audio_viewer) =
            PluginInput::new(delay_len, buffer_length, sample_rate, 128);
        let (sources_mutator, sources_viewer) = PluginSources::new(128, buffer_length);
        let input = LinearInterpolator::new(audio_viewer);
        let builder = RendererBuilder::new();
        let renderer = builder
            .set_source_interpreter(Interpreter::default())
            .set_sample_processor(processor)
            .set_audio_input(input)
            .set_spatial_input(sources_viewer)
            .set_audio_output(PluginOutput::new(2, buffer_length))
            .build();
        if renderer.is_err() {
            return false;
        }
        let handler = RenderHandler::new(renderer.unwrap(), audio_mutator, sources_mutator);
        self.render_handler = Some(handler);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.handle_distance_curve_change();
        self.handle_pan_law_change();

        let itd_nanos = self.itd_nanos();
        let handler = self.render_handler.as_mut().unwrap();
        let processor_mut: &mut Processor<_, _, _> = handler.renderer.sample_processor_mut();
        processor_mut.set_max_itd_nanos(itd_nanos);
        processor_mut.set_distance_effect(self.params.distance_effect.value());
        processor_mut.set_max_distance(self.params.max_distance.value());
        processor_mut.set_min_distance(self.params.min_distance.value());

        handler.render(buffer, context);
        ProcessStatus::Normal
    }
}

#[cfg(feature = "clap")]
impl ClapPlugin for HeadphoneMonitor {
    const CLAP_ID: &'static str = "audio.xpans.HeadphoneMonitor";

    const CLAP_DESCRIPTION: Option<&'static str> = None;

    const CLAP_MANUAL_URL: Option<&'static str> = None;

    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[];
}

#[cfg(feature = "vst3")]
impl Vst3Plugin for HeadphoneMonitor {
    const VST3_CLASS_ID: [u8; 16] = *b"HeadphoneMonitor";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Spatial,
        Vst3SubCategory::Stereo,
        Vst3SubCategory::Tools,
    ];
}

#[cfg(feature = "clap")]
nih_export_clap!(HeadphoneMonitor);

#[cfg(feature = "vst3")]
nih_export_vst3!(HeadphoneMonitor);
