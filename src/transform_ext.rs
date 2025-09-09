use bevy::prelude::*;

/// Extension trait for computing a camera's view matrix from its
/// `Transform`. Intended for use on camera transforms only.
pub trait CameraViewMatrix {
    // Calculate view_matrix (world -> view) for the camera's transform
    // NOTE: the view_matrix is not the same as the camera transforms
    // world -> local inverse because in view space the z axis is flipped.
    fn view_matrix(&self) -> Mat4;
}

impl CameraViewMatrix for Transform {
    fn view_matrix(&self) -> Mat4 {
        let camera_pos = self.translation;
        let camera_forward = self.forward();
        let target = camera_pos + camera_forward.as_vec3();

        Mat4::look_at_lh(camera_pos, target, Vec3::Y)
    }
}
