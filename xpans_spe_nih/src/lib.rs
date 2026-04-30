use nih_plug::{midi::NoteEvent, prelude::SysExMessage};
pub use xpans_spe_midi::spe;
use xpans_spe_midi::{read_message, spe::Message, write_message};

const BUF_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpeBundle {
    pub id: u16,
    pub msg: Message<f32>,
}

impl SpeBundle {
    #[inline]
    pub fn new(id: u16, msg: Message<f32>) -> Self {
        Self { id, msg }
    }
}

pub fn bundle_to_event(timing: u32, bundle: SpeBundle) -> NoteEvent<SpeBundle> {
    NoteEvent::MidiSysEx {
        timing,
        message: bundle,
    }
}

pub fn msg_to_event(timing: u32, id: u16, msg: Message<f32>) -> NoteEvent<SpeBundle> {
    let bundle = SpeBundle::new(id, msg);
    bundle_to_event(timing, bundle)
}

impl SysExMessage for SpeBundle {
    type Buffer = [u8; BUF_SIZE];

    #[inline]
    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        let (id, msg) = read_message(&buffer)?;
        Some(Self { id, msg })
    }
    #[inline]
    fn to_buffer(self) -> (Self::Buffer, usize) {
        let mut buffer = [0u8; BUF_SIZE];
        let writer = buffer.as_mut_slice();

        let len = write_message(writer, &self.msg, self.id);

        let written = BUF_SIZE - len;
        (buffer, written)
    }
}
