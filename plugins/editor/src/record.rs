use xpans_spe_nih::spe::{AxisCombo, Message};

use crate::params::PluginParams;

pub struct Record {
    pub pos: [f32; 3],
    pub ext: [f32; 3],
}

// impl Record {
//     pub fn pos_diff(&self, pos: [f32; 3]) -> Option<Message<f32>> {
//         cartestian_changes_msg(self.pos, pos, Message::pos)
//     }
//     pub fn ext_diff(&self, ext: [f32; 3]) -> Option<Message<f32>> {
//         cartestian_changes_msg(self.ext, ext, Message::ext)
//     }
// }

pub fn cartestian_changes_msg<T: PartialEq + Copy>(
    old: [T; 3],
    new: [T; 3],
    msg: fn(&AxisCombo, [T; 3]) -> Message<T>,
) -> Option<Message<T>> {
    let changes = changes(old, new);
    let axis = AxisCombo::from_changes(changes)?;
    Some(msg(&axis, new))
}

impl Default for Record {
    fn default() -> Self {
        let params = PluginParams::default();
        Self::from(&params)
    }
}

impl From<&PluginParams> for Record {
    fn from(value: &PluginParams) -> Self {
        Self {
            pos: value.pos(),
            ext: value.ext(),
        }
    }
}

fn changes<T, const N: usize>(lhs: [T; N], rhs: [T; N]) -> [bool; N]
where
    T: PartialEq,
{
    let mut result = [false; N];
    for (i, b) in result.iter_mut().enumerate() {
        *b = lhs[i] != rhs[i]
    }
    result
}
