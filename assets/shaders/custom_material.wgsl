#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::forward_io::VertexOutput

const MAX_STEPS: i32 = 100;
const HIT_THRESHOLD: f32 = 0.1;
const MAX_DISTANCE: f32 = 1000.0;

const RED: vec3<f32> = vec3(1.0, 0.0, 0.0);
const BLUE: vec3<f32> = vec3(0.0, 0.0, 1.0);
const WHITE: vec3<f32> = vec3(1.0, 1.0, 1.0);

struct GpuBox {
    position: vec3<f32>,
    size: f32, 
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> aspect_ratio: vec2<f32>;
@group(2) @binding(1)
var<uniform> camera_transform: mat4x4<f32>;
@group(2) @binding(2)
var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(3)
var<storage, read> boxes: array<GpuBox>;
@group(2) @binding(4)
var<storage, read_write> selection: array<f32>;

fn sdSphere(p: vec3<f32>, r: f32) -> SdfResult {
    let d = length(p) - r;
    return SdfResult(d, BLUE);
}

fn sdBox(p: vec3<f32>, b: vec3<f32>, color: vec3<f32>) -> SdfResult {
  let q = abs(p) - b;
  let d = length(max(q, vec3(0.0))) + min(max(q.x,max(q.y,q.z)), 0.0);
  return SdfResult(d, color);
}

fn sdGround(p: vec3<f32>) -> SdfResult {
  return SdfResult(-p.y, WHITE);
}

struct SdfResult {
    dist: f32,
    color: vec3<f32>,
}

fn map(p: vec3<f32>) -> SdfResult {
    var sdf =  sdGround(p);

    for (var i = 0u; i < arrayLength(&boxes); i++) {
        let box = boxes[i];
        let b = sdBox(p - box.position, vec3<f32>(box.size), box.color);

        sdf = min_sdf(sdf, b);
    }

    return sdf;
}

fn min_sdf(s1: SdfResult, s2: SdfResult) -> SdfResult {
    if (s1.dist < s2.dist) {
        return s1;
    };

    return s2;
}

fn ray_march(cameraOrigin: vec3<f32>, cameraDir: vec3<f32>) -> vec3<f32> {
    var dist = 0.0;

    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = cameraOrigin + dist * cameraDir;
        var transformed = camera_transform * vec4(pos, 1.0);

        let result = map(transformed.xyz / transformed.w);

        // Hit something
        if(result.dist < HIT_THRESHOLD) {
            return result.color;
        }

        dist = dist + result.dist;

        if(result.dist > MAX_DISTANCE) {
            break;
        }
    }

    // Outside max steps and didn't hit anything
    return vec3<f32>(0.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = (in.uv - 0.5) * aspect_ratio;
    let ray_dir = normalize(vec3<f32>(pixel_coords * 2 / aspect_ratio.y, 1.0));
    let ray_origin = vec3(0.0, 0.0, 0.0);

    let color = ray_march(ray_origin, ray_dir);

    let cursor_position_ndc = (cursor_position / aspect_ratio - 0.5) * aspect_ratio;


    if distance(pixel_coords, cursor_position_ndc) < 0.5 {
        selection[0] = color.x;
        selection[1] = color.y;
        selection[2] = color.z;
    }

    return vec4<f32>(color, 1.0);
}

fn remap(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (x - in_min) / (in_max - in_min);
    return t * (out_max - out_min) + out_min;
}

fn toSRGB(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 / 2.2));
}
