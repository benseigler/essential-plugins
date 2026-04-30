use xpans::{Extent, Position};
use xpans_spe::{SetExtent, SetPosition};

#[derive(Debug, Clone, Copy)]
pub struct Source<T> {
    pub pos_x: T,
    pub pos_y: T,
    pub pos_z: T,
    pub ext_x: T,
    pub ext_y: T,
    pub ext_z: T,
}

impl<T: Copy> Source<T> {
    pub fn pos(&self) -> [T; 3] {
        [self.pos_x, self.pos_y, self.pos_z]
    }
}

impl Default for Source<f32> {
    fn default() -> Self {
        Self {
            pos_x: 0.,
            pos_y: 1.,
            pos_z: 0.,
            ext_x: 0.,
            ext_y: 0.,
            ext_z: 0.,
        }
    }
}

impl<T: Copy> Extent<T> for Source<T> {
    fn ext_x(&self) -> T {
        self.ext_x
    }

    fn ext_y(&self) -> T {
        self.ext_y
    }

    fn ext_z(&self) -> T {
        self.ext_z
    }
}

impl<T: Copy> Position<T> for Source<T> {
    fn pos_x(&self) -> T {
        self.pos_x
    }

    fn pos_y(&self) -> T {
        self.pos_y
    }

    fn pos_z(&self) -> T {
        self.pos_z
    }
}

impl<V> SetPosition<V> for Source<V> {
    fn set_pos_x(&mut self, x: V) {
        self.pos_x = x
    }

    fn set_pos_y(&mut self, y: V) {
        self.pos_y = y
    }

    fn set_pos_z(&mut self, z: V) {
        self.pos_z = z
    }
}

impl<V> SetExtent<V> for Source<V> {
    fn set_ext_x(&mut self, x: V) {
        self.ext_x = x
    }

    fn set_ext_y(&mut self, y: V) {
        self.ext_y = y
    }

    fn set_ext_z(&mut self, z: V) {
        self.ext_z = z
    }
}

// const SIMD_WIDTH: usize = 4;

// fn vector_index(id: usize) -> usize {
//     id / SIMD_WIDTH
// }
