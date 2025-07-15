use bevy::prelude::*;

use crate::camera;

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalId::default())
            .add_systems(Update, place_box_system);
    }
}

#[derive(Component)]
pub struct BoxGeometry {
    pub position: Vec3,
    pub size: f32,
    pub id: u32,
}

#[derive(Resource, Default)]
struct GlobalId(u32);

impl GlobalId {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;

        id
    }
}

impl BoxGeometry {
    fn new(position: Vec3, id: u32) -> Self {
        BoxGeometry {
            position,
            size: 1.0,
            id,
        }
    }
}

fn place_box_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Res<camera::CameraControls>,
    mut global_id: ResMut<GlobalId>,
    mut commands: Commands,
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

    commands.spawn(BoxGeometry::new(hit, global_id.next()));
}
