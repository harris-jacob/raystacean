use bevy::prelude::*;
use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(u32);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn to_color(self) -> [f32; 3] {
        let r = ((self.0 & 0xFF) as f32) / 255.0;
        let g = (((self.0 >> 8) & 0xFF) as f32) / 255.0;
        let b = (((self.0 >> 16) & 0xFF) as f32) / 255.0;

        [r, g, b]
    }

    pub fn to_scrambled_color(self) -> [f32; 3] {
        let id = scramble(self.0);

        let r = ((id & 0xFF) as f32) / 255.0;
        let g = (((id >> 8) & 0xFF) as f32) / 255.0;
        let b = (((id >> 16) & 0xFF) as f32) / 255.0;

        [r, g, b]
    }

    pub fn from_color(color: Vec3) -> Self {
        let r = (color.x * 255.0) as u32;
        let g = ((color.y * 255.0) as u32) << 8;
        let b = ((color.z * 255.0) as u32) << 16;

        Self(r | g | b)
    }
}

const MODULUS: u32 = 1 << 24;
const MULTIPLIER: u32 = 0x00C297D7;
const XOR_MASK: u32 = 0x0055AA33;

fn scramble(x: u32) -> u32 {
    assert!(x < MODULUS);
    let x = x ^ XOR_MASK;
    x.wrapping_mul(MULTIPLIER) & 0xFFFFFF
}
