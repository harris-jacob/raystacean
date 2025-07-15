use bevy::{prelude::*, render::storage::ShaderStorageBuffer};

use crate::{camera, rendering::GpuBox};


#[derive(Component)]
pub struct BoxGeometry {
    pub position: Vec3,
    pub size: f32,
}

impl Default for BoxGeometry {
    fn default() -> Self {
        BoxGeometry {
            position: Vec3::new(0.0, 0.0, -2.0),
            size: 1.0,
        }
    }
}

impl BoxGeometry {
    fn with_position(self, position: Vec3) -> Self {
        BoxGeometry { position, ..self }
    }
}

fn place_box_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Res<camera::CameraControls>,
    // TODO: this is rendering
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single().expect("single");


    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // TODO: tidy this math up and write some comments
    let screen_size = window.size();
    let ndc = (cursor_pos / screen_size) * 2.0 - Vec2::ONE;
    let pixel_coords = ndc * Vec2::new(window.width() / window.height(), 1.0);

    let ray_dir_camera_space = Vec3::new(pixel_coords.x, pixel_coords.y, 1.0).normalize();

    let camera_inv = camera.transform().inverse();

    let ray_dir = (camera_inv * ray_dir_camera_space.extend(0.0))
        .truncate()
        .normalize();
    let ray_origin = (camera_inv * Vec4::new(0.0, 0.0, 0.0, 1.0)).truncate();

    // Intersect with ground plane (Y=0)
    let t = -ray_origin.y / ray_dir.y;

    if t < 0.0 {
        return;
    }

    let hit = ray_origin + ray_dir * t;

    box_state.0.push(new_box);

    buffer.set_data(box_state.0.clone());
}


impl From<BoxGeometry> for GpuBox {
    fn from(value: BoxGeometry) -> Self {
        GpuBox {
            position: todo!(),
            size: todo!(),
            color: todo!(),
            // _padding: todo!(),
        }
    }
}
