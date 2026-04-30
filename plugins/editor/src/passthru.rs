use nih_plug::prelude::*;
use xpans_spe_nih::SpeBundle;

#[inline]
pub fn spe_editor_passthru<C, P>(context: &mut C, editing_ids: &[u16])
where
    P: Plugin<SysExMessage = SpeBundle>,
    C: ProcessContext<P>,
{
    while let Some(event) = context.next_event() {
        if let NoteEvent::MidiSysEx {
            timing: _timing,
            message,
        } = event
        {
            let mut collision = false;
            for id in editing_ids {
                collision |= *id == message.id;
            }
            if collision == false {
                context.send_event(event);
            }
        }
    }
}
