use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct PixelColorUnderCursor(Vec3);

#[derive(Event, Debug)]
pub struct PlaneClicked;

#[derive(Event, Debug)]
pub struct OriginDragged {
    pub axis: Vec3,
    pub delta: f32,
}

impl OriginDragged {
    pub fn new(axis: Vec3, delta: f32) -> Self {
        Self { axis, delta }
    }
}

impl PixelColorUnderCursor {
    pub fn new(color: Vec3) -> Self {
        Self(color)
    }

    pub fn color(&self) -> Vec3 {
        self.0
    }
}
