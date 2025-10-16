mod camera;
mod controls;
mod events;
mod geometry;
mod gizmos;
mod global_id;
mod layers;
mod manipulation;
mod node_id;
mod rendering;
mod selection;
mod transform_ext;
mod ui;

use std::path::PathBuf;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [99]))]
fn main() {
    let asset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .to_string_lossy()
        .to_string();

    App::new()
        .add_event::<events::PixelColorUnderCursor>()
        .add_event::<events::PlaneClicked>()
        .add_event::<events::OriginDragged>()
        .add_event::<events::GeometryAdded>()
        .add_event::<events::ScalingGizmoDragged>()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: asset_path,
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin {
            smoothing_factor: 0.9,
            ..default()
        })
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            camera::CameraPlugin,
            controls::ControlContextPlugin,
            geometry::GeometryPlugin,
            gizmos::GizmosPlugin,
            global_id::GlobalIdPlugin,
            manipulation::ManipulationPlugin,
            rendering::RenderingPlugin,
            selection::SelectionPlugin,
            ui::UiPlugin,
        ))
        .run();
}
