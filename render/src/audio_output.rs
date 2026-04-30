use std::ops::AddAssign;

use xpans_violet::{Connector, audio_output::AudioOutput};

pub struct PluginOutput<T> {
    buffer: Box<[T]>,
    buffer_length: usize,
    channels: usize,
}

impl<T: Default + Copy> PluginOutput<T> {
    pub fn new(channels: usize, buffer_length: usize) -> Self {
        let buffer = vec![T::default(); channels * buffer_length].into_boxed_slice();
        Self {
            buffer,
            buffer_length,
            channels,
        }
    }
    pub fn index(&self, frame: usize, channel: usize) -> usize {
        (frame * self.channels) + channel
    }
    pub fn get_sample(&self, frame: usize, channel: usize) -> T {
        let index = self.index(frame, channel);
        self.buffer[index]
    }
    pub fn clear(&mut self) {
        for sample in self.buffer.iter_mut() {
            *sample = T::default()
        }
    }
}
impl<T> Connector for PluginOutput<T> {
    fn advance(&mut self, _frames: usize) {}

    fn frames_available(&self) -> Option<usize> {
        Some(self.buffer_length)
    }
}
impl<T: AddAssign + Default + Copy> AudioOutput for PluginOutput<T> {
    type Sample = T;

    fn set_sample(&mut self, channel: usize, frame: usize, value: Self::Sample) {
        let index = self.index(frame, channel);
        self.buffer[index] += value
    }

    fn channel_count(&self) -> usize {
        self.channels
    }
}
