use std::marker::PhantomData;

use num::Float;

use wreath::{Reader, RingReader, RingWriter, Writer, ring_buf};
use xpans_spe::ApplyMessage;
use xpans_spe::Message;
use xpans_violet::spatial_input::SpatialInput;
use xpans_violet::{Connector, Source};

pub struct PluginSources<T> {
    phantom_data: PhantomData<T>,
}

pub struct SourcesViewer<T> {
    sources: RingReader<Source<T>>,
    source_count: usize,
}

impl<T> SourcesViewer<T> {
    pub fn new(sources: RingReader<Source<T>>, source_count: usize) -> Self {
        Self {
            sources,
            source_count,
        }
    }
}

pub struct SourcesMutator<T> {
    sources: RingWriter<Source<T>>,
    current: Box<[Source<T>]>,
    source_count: usize,
}

// If we were to use the usual default value (all zeroes), we would get NaN
// samples in some renderers.
fn default_source() -> Source<f32> {
    Source {
        pos_x: 0.,
        pos_y: 1.,
        pos_z: 0.,
        ext_x: 0.,
        ext_y: 0.,
        ext_z: 0.,
    }
}
impl SourcesMutator<f32> {
    pub fn new(sources: RingWriter<Source<f32>>, source_count: usize) -> Self {
        Self {
            sources,
            current: vec![default_source(); source_count].into_boxed_slice(),
            source_count,
        }
    }
    pub fn apply_msg(&mut self, id: usize, msg: Message<f32>) {
        self.current.get_mut(id).unwrap().apply_message(msg)
    }
    pub fn write_current_sources(&mut self, frame: usize) {
        for (i, source) in self.current.iter().enumerate() {
            self.sources
                .write_forward((self.source_count * frame) + i, *source);
        }
    }
    pub fn move_write_position(&mut self, frames: usize) {
        self.sources
            .advance_write_position_by(frames * self.source_count);
    }
}

impl PluginSources<f32> {
    pub fn new(sources: usize, buffer_length: usize) -> (SourcesMutator<f32>, SourcesViewer<f32>) {
        let (reader, writer) = ring_buf(1, buffer_length * sources);
        (
            SourcesMutator::new(writer, sources),
            SourcesViewer::new(reader, sources),
        )
    }
}
impl<T: Float + Copy + Default> SpatialInput for SourcesViewer<T> {
    type Scalar = T;
    fn source(&self, source: usize, frame: usize) -> Source<T> {
        self.sources
            .read_forward((self.source_count * frame) + source)
    }
    fn source_count(&self) -> usize {
        self.source_count
    }
}

impl<T: Copy + Default> Connector for SourcesViewer<T> {
    fn advance(&mut self, frames: usize) {
        self.sources
            .advance_read_position_by(frames * self.source_count);
    }

    fn frames_available(&self) -> Option<usize> {
        Some(self.sources.real_reads_available() / self.source_count)
    }
}
