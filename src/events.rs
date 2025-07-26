use bevy::prelude::*;

#[derive(Event)]
pub struct PixelColorUnderCursor(Vec3);

impl PixelColorUnderCursor {
    pub fn new(color: Vec3) -> Self {
        Self(color)
    }

    pub fn color(&self) -> Vec3 {
        self.0
    }
}
