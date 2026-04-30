use std::marker::PhantomData;

use nih_plug::prelude::*;
use xpans_common_lr::{Linear, PanLaw, Sine, SquareRoot};

pub const PASSTHRU_LAYOUT: AudioIOLayout = AudioIOLayout {
    main_input_channels: None,
    main_output_channels: None,
    aux_input_ports: &[],
    aux_output_ports: &[],
    names: PortNames::const_default(),
};

pub const STEREO: AudioIOLayout = AudioIOLayout {
    main_input_channels: Some(unsafe { NonZeroU32::new_unchecked(2) }),
    main_output_channels: Some(unsafe { NonZeroU32::new_unchecked(2) }),
    aux_output_ports: &[],
    aux_input_ports: &[],
    names: PortNames {
        layout: None,
        main_input: None,
        main_output: None,
        aux_inputs: &[],
        aux_outputs: &[],
    },
};

#[inline]
pub fn events<'a, C, P>(context: &'a mut C) -> Events<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    Events::new(context)
}

#[inline]
pub fn sysex<'a, C, P>(context: &'a mut C) -> SysExMessages<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    SysExMessages::new(context)
}

pub struct Events<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    context: &'a mut C,
    phantom_data: PhantomData<P>,
}
pub struct SysExMessages<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    context: &'a mut C,
    phantom_data: PhantomData<P>,
}

impl<'a, C, P> SysExMessages<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    #[inline]
    fn new(context: &'a mut C) -> Self {
        Self {
            context,
            phantom_data: PhantomData,
        }
    }
}
impl<'a, C, P> Iterator for SysExMessages<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    type Item = (u32, P::SysExMessage);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for event in events(self.context) {
            match event {
                NoteEvent::MidiSysEx { timing, message } => return Some((timing, message)),
                _ => continue,
            };
        }
        None
    }
}

impl<'a, C, P> Events<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    #[inline]
    fn new(context: &'a mut C) -> Self {
        Self {
            context,
            phantom_data: PhantomData,
        }
    }
}
impl<'a, C, P> Iterator for Events<'a, C, P>
where
    C: ProcessContext<P>,
    P: Plugin,
{
    type Item = NoteEvent<P::SysExMessage>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.context.next_event()
    }
}

#[inline]
pub fn passthrough_events<C: ProcessContext<P>, P: Plugin>(context: &mut C) {
    while let Some(event) = context.next_event() {
        context.send_event(event);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum PanLawOption {
    #[name = "-6dB Linear"]
    Linear,
    #[name = "-3dB Sine"]
    Sine,
    #[name = "-3dB Square Root"]
    SquareRoot,
}

impl Default for PanLawOption {
    fn default() -> Self {
        Self::Sine
    }
}

impl PanLawOption {
    pub fn get_dyn(&self) -> Box<dyn PanLaw<f32> + Send> {
        match self {
            Self::Linear => Box::new(Linear),
            Self::Sine => Box::new(Sine),
            Self::SquareRoot => Box::new(SquareRoot),
        }
    }
}
