use serde::{Deserialize, Serialize};

use crate::voxel::Voxel;

#[repr(u32)]
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Block {
    #[default]
    Air = 0,
    Stone,
    Dirt,
    Grass,
    // Partial(u8),
}

impl Block {
    const ALL: &'static [Self] = &[Self::Air, Self::Stone, Self::Dirt, Self::Grass];
}

impl Voxel for Block {
    type Raw = u32;

    fn default_empty() -> Self {
        Self::Air
    }

    fn default_opaque() -> Self {
        Self::Stone
    }

    fn is_opaque(&self) -> bool {
        match self {
            Self::Air => false,
            // Self::Partial(_) => false,
            _ => true,
        }
    }

    fn lerp(a: Self, b: Self, t: f64) -> Self {
        if t < 0.5 {
            a
        } else {
            b
        }
    }

    fn raw(&self) -> Self::Raw {
        *self as Self::Raw
    }

    fn all() -> &'static [Self] {
        Self::ALL
    }
}
