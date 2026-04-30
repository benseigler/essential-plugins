use std::collections::BTreeMap;
mod audio_input;
mod audio_output;
mod spatial_input;

use nih_plug::prelude::*;
use shared::sysex;
use xpans_spe_nih::SpeBundle;
use xpans_violet::{
    Renderer, SampleProcessor, SourceInterpreter, audio_input::AudioInput,
    audio_output::AudioOutput,
};

pub use crate::spatial_input::{PluginSources, SourcesMutator, SourcesViewer};

pub use {
    audio_input::{PluginInput, PluginInputMutator, PluginInputViewer},
    audio_output::PluginOutput,
};

pub struct RenderHandler<Interpreter, Processor, AudioIn>
where
    AudioIn: AudioInput<Sample = f32>,
    Interpreter: SourceInterpreter<f32, Interpretation = Processor::Interpretation>,
    Interpreter::Interpretation: Default,
    Processor: SampleProcessor<AudioIn, PluginOutput<f32>>,
{
    pub timing_map: BTreeMap<u32, Vec<SpeBundle>>,
    pub renderer: Renderer<Interpreter, Processor, AudioIn, SourcesViewer<f32>, PluginOutput<f32>>,
    pub audio_mutator: PluginInputMutator<f32>,
    pub sources_mutator: SourcesMutator<f32>,
}

impl<Interpreter, Processor, AudioIn> RenderHandler<Interpreter, Processor, AudioIn>
where
    AudioIn: AudioInput<Sample = f32>,
    Interpreter: SourceInterpreter<f32, Interpretation = Processor::Interpretation>,
    Interpreter::Interpretation: Default + Clone,
    Processor: SampleProcessor<AudioIn, PluginOutput<f32>>,
{
    pub fn new(
        renderer: Renderer<Interpreter, Processor, AudioIn, SourcesViewer<f32>, PluginOutput<f32>>,
        audio_mutator: PluginInputMutator<f32>,
        sources_mutator: SourcesMutator<f32>,
    ) -> Self {
        Self {
            timing_map: BTreeMap::new(),
            renderer,
            audio_mutator,
            sources_mutator,
        }
    }
    pub fn render<P>(&mut self, buffer: &mut Buffer, context: &mut impl ProcessContext<P>)
    where
        P: Plugin<SysExMessage = SpeBundle>,
    {
        for (message_sample_offset, bundle) in sysex(context) {
            self.timing_map
                .entry(message_sample_offset)
                .or_default()
                .push(bundle);
        }
        for relative_frame in 0..buffer.samples() {
            if let Some(bundles) = self.timing_map.get(&(relative_frame as u32)) {
                for bundle in bundles.iter() {
                    let id = bundle.id as usize;
                    self.sources_mutator.apply_msg(id, bundle.msg);
                }
            }
            self.sources_mutator.write_current_sources(relative_frame);
            for channel in 0..buffer.channels() {
                let sample = buffer.as_slice_immutable()[channel][relative_frame];
                self.audio_mutator
                    .set_sample(relative_frame, channel, sample);
            }
        }
        self.audio_mutator.move_write_position(buffer.samples());
        self.sources_mutator.move_write_position(buffer.samples());
        self.renderer.render_available_frames();

        let output_channels = self.renderer.audio_output().channel_count();
        let audio_output_mut = self.renderer.audio_output_mut();
        for relative_frame in 0..buffer.samples() {
            for channel in 0..output_channels {
                let sample = audio_output_mut.get_sample(relative_frame, channel);
                buffer.as_slice()[channel][relative_frame] = sample;
            }
        }
        self.renderer.audio_output_mut().clear();
        self.timing_map.clear();
    }
    pub fn renderer_mut(
        &mut self,
    ) -> &mut Renderer<Interpreter, Processor, AudioIn, SourcesViewer<f32>, PluginOutput<f32>> {
        &mut self.renderer
    }
}
