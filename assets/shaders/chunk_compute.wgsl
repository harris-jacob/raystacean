@group(0) @binding(0)
var sdf_output: texture_storage_3d<rgba32float, write>;

@group(0) @binding(1)
var<uniform> chunk_index: vec3<u32>;

const CHUNK_RESOLUTION: u32 = 32u;
// CHUNK_RESOLUTION / CHUNK_SIZE
const VOXEL_PITCH: f32 = 0.5;
// NUMBER_OF_CHUNKS/AXIS * CHUNK_RES
const TEXTURE_RES: u32 = 512;
const WORLD_SIZE: f32 = 256.0;

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (any(id >= vec3<u32>(CHUNK_RESOLUTION))) {
        return;
    }

    let global_voxel = chunk_index * u32(CHUNK_RESOLUTION) + id;

    let world_pos =
        ((vec3<f32>(global_voxel) + 0.5) / f32(TEXTURE_RES)) * WORLD_SIZE
        - WORLD_SIZE * 0.5;

    let dist = length(world_pos) - 1.0;

    textureStore(sdf_output, global_voxel, vec4<f32>(0.0, 1.0, 0.0, dist));
}
