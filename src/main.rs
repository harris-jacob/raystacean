mod camera;
mod controls;
mod events;
mod geometry;
mod gizmos;
mod global_id;
mod layers;
mod manipulation;
mod operations;
mod rendering;
mod selection;
mod transform_ext;
mod ui;
mod node_id;

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_event::<events::PixelColorUnderCursor>()
        .add_event::<events::PlaneClicked>()
        .add_event::<events::OriginDragged>()
        .add_event::<events::GeometryAdded>()
        .add_event::<events::UnionOperationPerformed>()
        .add_event::<events::UnionOperationErrored>()
        .add_event::<events::ScalingGizmoDragged>()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::DEBUG,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            camera::CameraPlugin,
            controls::ControlContextPlugin,
            geometry::GeometryPlugin,
            gizmos::GizmosPlugin,
            global_id::GlobalIdPlugin,
            manipulation::ManipulationPlugin,
            operations::OperationsPlugin,
            rendering::RenderingPlugin,
            selection::SelectionPlugin,
            ui::UiPlugin,
        ))
        .run();
}
