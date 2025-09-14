use bevy::{prelude::*, render::camera::CameraProjection};

use crate::{camera, controls, events, transform_ext::CameraViewMatrix};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalId::default());

        app.add_observer(place_box);
    }
}

#[derive(Component, Debug)]
pub struct BoxGeometry {
    pub position: Vec3,
    pub scale: Vec3,
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
            scale: Vec3::ONE * 2.5,
            id: GeometryId::new(id),
        }
    }

    fn with_y(self, y: f32) -> Self {
        Self {
            position: self.position.with_y(y),
            ..self
        }
    }
}

fn place_box(
    _trigger: Trigger<events::PlaneClicked>,
    control_mode: Res<controls::ControlMode>,
    windows: Query<&Window>,
    camera: Query<(&Projection, &Transform), With<camera::MainCamera>>,
    mut global_id: ResMut<GlobalId>,
    mut commands: Commands,
) {
    if *control_mode != controls::ControlMode::PlaceGeometry {
        return;
    }

    let window = windows.single().expect("single");
    let (projection, transform) = camera.single().expect("single");

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    if let Some(hit) = cast_ray_at_ground_in_scene(cursor_pos, projection, transform, window) {
        let geometry = BoxGeometry::new(hit, global_id.next());
        // sit the box on the  plane rather than putting the center on it
        let y = geometry.scale.y;
        commands.spawn(geometry.with_y(y));
    };
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

// Given a 2d screen space position, fire a ray into the scene along the camera
// axis and find the intersection with the ground plane (Y=0). Returns `None`
// if there is no intersection (e.g.) ray direction points away from ground plane.
fn cast_ray_at_ground_in_scene(
    screen_space_position: Vec2,
    projection: &Projection,
    camera_transform: &Transform,
    window: &Window,
) -> Option<Vec3> {
    let screen_size = window.size();

    let ndc = (screen_space_position / screen_size) * 2.0 - Vec2::ONE;
    let ndc = Vec4::new(ndc.x, ndc.y, -1.0, 1.0);

    // 2. NDC → view space
    let view_pos_h = projection.get_clip_from_view().inverse() * ndc;
    let view_pos = view_pos_h.xyz() / view_pos_h.w;

    // Ray in view space
    let ray_origin_view = Vec4::default().with_w(1.0);
    let ray_dir_view = view_pos.normalize().extend(0.0);

    // 4. View -> world
    let ray_origin_world = (camera_transform.view_matrix().inverse() * ray_origin_view).xyz();

    let ray_dir_world = (camera_transform.view_matrix().inverse() * ray_dir_view)
        .normalize()
        .xyz();

    // Solve for t value that intersects XZ plane (Y=0)
    let t = -ray_origin_world.y / ray_dir_world.y;

    // Intersection point is behind origin
    if t < 0.0 {
        return None;
    }

    // Substitute back into ray eqn
    Some(ray_origin_world + ray_dir_world * t)
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
