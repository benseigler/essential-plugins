use std::marker::PhantomData;

use wreath::{MultiRingReader, MultiRingWriter, Reader, Writer, multi_ring_buf};
use xpans_violet::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput},
};

pub struct PluginInput<T> {
    phantom_data: PhantomData<T>,
}
impl<T: Default + Copy> PluginInput<T> {
    pub fn new(
        buffer_size: usize,
        write_capacity: usize,
        sample_rate: u32,
        channels: usize,
    ) -> (PluginInputMutator<T>, PluginInputViewer<T>) {
        let (reader, writer) = multi_ring_buf(channels, buffer_size, write_capacity);
        (
            PluginInputMutator::new(writer),
            PluginInputViewer::new(reader, sample_rate, channels),
        )
    }
}

pub struct PluginInputViewer<T> {
    input: MultiRingReader<T>,
    sample_rate: u32,
    channels: usize,
}

impl<T> PluginInputViewer<T> {
    pub fn new(input: MultiRingReader<T>, sample_rate: u32, channels: usize) -> Self {
        Self {
            input,
            sample_rate,
            channels,
        }
    }
}

impl<T: Copy + Default> Connector for PluginInputViewer<T> {
    fn advance(&mut self, frames: usize) {
        self.input.advance_read_position_by(frames);
    }

    fn frames_available(&self) -> Option<usize> {
        Some(self.input.real_reads_available())
    }
}
impl<T: Copy + Default> AudioInput for PluginInputViewer<T> {
    type Sample = T;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.input.read_forward(channel, frame)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channel_count(&self) -> usize {
        self.channels
    }
}
impl<T: Copy + Default> BufferedAudioInput for PluginInputViewer<T> {
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        let frame = frame.cast_signed();
        let index = frame.saturating_sub_unsigned(sample);
        self.input.read_relative(channel, index)
    }
}

pub struct PluginInputMutator<T> {
    input: MultiRingWriter<T>,
}

impl<T: Copy + Default> PluginInputMutator<T> {
    pub fn new(input: MultiRingWriter<T>) -> Self {
        Self { input }
    }
    pub fn set_sample(&mut self, frame: usize, channel: usize, sample: T) {
        self.input.write_forward_unchecked(channel, frame, sample);
    }
    pub fn move_write_position(&self, frames: usize) {
        self.input.advance_write_position_by(frames);
    }
}
