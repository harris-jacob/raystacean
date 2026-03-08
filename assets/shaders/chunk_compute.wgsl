#import "./shaders/sdf.wgsl"::{sd_sphere, sd_box, min_sdf, max_sdf, SdfResult}

@group(0) @binding(0)
var sdf_output: texture_storage_3d<rgba32float, write>;
@group(0) @binding(1)
var<uniform> chunk_index: vec3<u32>;
@group(0) @binding(2)
var<storage, read> primatives: array<GpuPrimative>;
@group(0) @binding(3)
var<uniform> primatives_meta: vec4<u32>; // only x used = count

struct GpuPrimative {
    position: vec3<f32>,
    is_subtract: u32,
    scale: vec3<f32>, 
    blend: f32,
    color: vec3<f32>,
    rounding: f32,
    logical_color: vec3<f32>,
}

const CHUNK_RESOLUTION: u32 = 32u;
// CHUNK_RESOLUTION / CHUNK_SIZE
const VOXEL_PITCH: f32 = 0.5;
// NUMBER_OF_CHUNKS/AXIS * CHUNK_RES
const TEXTURE_RES: u32 = 512;
const WORLD_SIZE: f32 = 256.0;

const BLACK: vec3<f32> = vec3(0.0, 0.0, 0.0);

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (any(id >= vec3<u32>(CHUNK_RESOLUTION))) {
        return;
    }

    let global_voxel = chunk_index * u32(CHUNK_RESOLUTION) + id;

    let world_pos =
        ((vec3<f32>(global_voxel) + 0.5) / f32(TEXTURE_RES)) * WORLD_SIZE
        - WORLD_SIZE * 0.5;

    var sdf = SdfResult(100.0, BLACK);
    for (var i = 0u; i < primatives_meta.x; i++) {
        let box = primatives[i];

        let color = box.color;
        let b = sd_box(world_pos - box.position, box.scale, box.rounding, color);

        if (box.is_subtract == 1u) {
            sdf.dist = op_smooth_subtract(b.dist, sdf.dist, box.blend);
        } else {
            sdf = sd_smooth_union(sdf, b, box.blend);
        }
    }

    textureStore(sdf_output, global_voxel, vec4<f32>(0.0, 1.0, 0.0, sdf.dist));
}

// Cubic polynomial adapted from: https://iquilezles.org/articles/smin/
fn op_smooth_subtract(s1: f32, s2: f32, k: f32) -> f32 {
    let n = abs(s1 + s2) / (6.0 * k);
    let h = 1.0 - min(n, 1.0);
    let w = h * h * h;
    let s = w * k;

    return -min(s1, -s2) + s;
}

// cubic polynomial with color blending. Taken from:
// https://iquilezles.org/articles/smin/
fn sd_smooth_union(s1: SdfResult, s2: SdfResult, k: f32) -> SdfResult {
    let n = abs(s1.dist - s2.dist) / (6.0 * k);
    let h = 1.0 - min(n, 1.0);
    let w = h * h * h;
    let s = w * k;
    let m = w * 0.5;

    if (s1.dist < s2.dist) {
        let c = mix(s1.color, s2.color, m);
        return SdfResult(s1.dist - s, c);
    } else {
        let c = mix(s1.color, s2.color, 1.0 - m);
        return SdfResult(s2.dist - s, c);
    }
}

