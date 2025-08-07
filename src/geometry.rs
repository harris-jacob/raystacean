use bevy::prelude::*;

use crate::{camera, controls, events};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalId::default());

        app.add_observer(place_box);
    }
}

#[derive(Component)]
pub struct BoxGeometry {
    pub position: Vec3,
    pub size: f32,
    pub id: GeometryId,
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
            id: GeometryId::new(id),
        }
    }
}

fn place_box(
    _trigger: Trigger<events::PlaneClicked>,
    control_mode: Res<controls::ControlMode>,
    windows: Query<&Window>,
    camera: Res<camera::CameraControls>,
    mut global_id: ResMut<GlobalId>,
    mut commands: Commands,
) {
    if *control_mode != controls::ControlMode::PlaceGeometry {
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

    let mut hit = ray_origin + ray_dir * t;

    // sit the box on the plane rather than putting the center on it
    hit.y -= 1.0;

    commands.spawn(BoxGeometry::new(hit, global_id.next()));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryId(u32);

impl GeometryId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn to_color(self) -> [f32; 3] {
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

        let id = unscramble(r | g | b);

        Self(id)
    }
}

const MODULUS: u32 = 1 << 24;
const MULTIPLIER: u32 = 0x00C297D7;
const INV_MULTIPLIER: u32 = 0xDB4BE7;
const XOR_MASK: u32 = 0x0055AA33;

fn scramble(x: u32) -> u32 {
    assert!(x < MODULUS);
    let x = x ^ XOR_MASK;
    x.wrapping_mul(MULTIPLIER) & 0xFFFFFF
}

fn unscramble(x: u32) -> u32 {
    let x = x.wrapping_mul(INV_MULTIPLIER) & 0xFFFFFF;
    x ^ XOR_MASK
}
