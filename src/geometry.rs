use bevy::{prelude::*, render::camera::CameraProjection};

use crate::{camera, controls, events, global_id, node_id, transform_ext::CameraViewMatrix};

pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(place_box);
    }
}

#[derive(Component, Debug)]
pub struct BoxGeometry {
    pub position: Vec3,
    pub scale: Vec3,
    pub color: [f32; 3],
    pub rounding: f32,
    pub id: node_id::NodeId,
}

impl BoxGeometry {
    fn new(position: Vec3, id: u32) -> Self {
        let id = node_id::NodeId::new(id);
        BoxGeometry {
            position,
            scale: Vec3::ONE * 2.5,
            rounding: 0.0,
            color: id.to_scrambled_color(),
            id,
        }
    }

    fn with_y(self, y: f32) -> Self {
        Self {
            position: self.position.with_y(y),
            ..self
        }
    }

    /// Compute rounding radius using the rounding factor and scale
    /// of the smallest axis.
    pub fn rounding_radius(&self) -> f32 {
        self.rounding * self.scale.x.min(self.scale.y).min(self.scale.z)
    }
}

fn place_box(
    _trigger: Trigger<events::PlaneClicked>,
    mut control_mode: ResMut<controls::ControlMode>,
    windows: Query<&Window>,
    camera: Query<(&Projection, &Transform), With<camera::MainCamera>>,
    mut global_id: ResMut<global_id::GlobalId>,
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

        let geometry_id = geometry.id;

        let entity_id = commands.spawn(geometry.with_y(y)).id();

        commands.trigger(events::GeometryAdded {
            id: geometry_id,
            entity: entity_id,
        });

        *control_mode = controls::ControlMode::Select;
    };
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
