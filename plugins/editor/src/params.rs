use nih_plug::prelude::*;
#[derive(Params)]
pub struct PluginParams {
    #[id = "Source ID"]
    pub id: IntParam,
    #[id = "Position X"]
    pub pos_x: FloatParam,
    #[id = "Position Y"]
    pub pos_y: FloatParam,
    #[id = "Position Z"]
    pub pos_z: FloatParam,
    #[id = "Extent X"]
    pub ext_x: FloatParam,
    #[id = "Extent Y"]
    pub ext_y: FloatParam,
    #[id = "Extent Z"]
    pub ext_z: FloatParam,
}

impl PluginParams {
    pub fn id(&self) -> u16 {
        self.id.value() as u16
    }
    pub fn pos(&self) -> [f32; 3] {
        [self.pos_x.value(), self.pos_y.value(), self.pos_z.value()]
    }
    pub fn ext(&self) -> [f32; 3] {
        [self.ext_x.value(), self.ext_y.value(), self.ext_z.value()]
    }
}
impl Default for PluginParams {
    fn default() -> Self {
        Self {
            id: IntParam::new(
                "Source ID",
                0,
                IntRange::Linear {
                    min: 0,
                    max: 2_i32.pow(14),
                },
            ),
            pos_x: FloatParam::new("Position X", 0.0, FloatRange::Linear { min: -1., max: 1. }),
            pos_y: FloatParam::new("Position Y", 0.0, FloatRange::Linear { min: -1., max: 1. }),
            pos_z: FloatParam::new("Position Z", 0.0, FloatRange::Linear { min: -1., max: 1. }),
            ext_x: FloatParam::new("Extent X", 0.0, FloatRange::Linear { min: 0., max: 2. }),
            ext_y: FloatParam::new("Extent Y", 0.0, FloatRange::Linear { min: 0., max: 2. }),
            ext_z: FloatParam::new("Extent Z", 0.0, FloatRange::Linear { min: 0., max: 2. }),
        }
    }
}
