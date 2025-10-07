use bevy::prelude::*;

pub struct GlobalIdPlugin;

impl Plugin for GlobalIdPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalId::default());
    }
}

#[derive(Resource, Default)]
pub struct GlobalId(u32);

impl GlobalId {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;

        id
    }
}
