#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::forward_io::VertexOutput

const MAX_STEPS: i32 = 256;
const HIT_THRESHOLD: f32 = 0.001;
const MAX_DISTANCE: f32 = 1000.0;

const RED: vec3<f32> = vec3(1.0, 0.0, 0.0);
const BLUE: vec3<f32> = vec3(0.0, 0.0, 1.0);
const WHITE: vec3<f32> = vec3(1.0, 1.0, 1.0);

@group(2) @binding(0)
var<uniform> aspect_ratio: vec2<f32>;
@group(2) @binding(1)
var<uniform> camera_rotation: mat3x3<f32>;

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
    var sdf =  sdBox(p - vec3<f32>(-2.0, 0, 0.0), vec3<f32>(1.0), BLUE);
    sdf =  min_sdf(sdf, sdBox(p - vec3<f32>(2.0, 0, 0.0), vec3<f32>(1.0), RED));
    sdf =  min_sdf(sdf, sdGround(p - vec3<f32>(0.0, 2.0, 0.0)));

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
        pos = camera_rotation * pos;

        let result = map(pos);

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
    let ray_origin = vec3(0.0, 0.0, -5.0);

    let color = ray_march(ray_origin, ray_dir);

    return vec4<f32>(color, 1.0);
}

fn remap(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (x - in_min) / (in_max - in_min);
    return t * (out_max - out_min) + out_min;
}

fn toSRGB(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 / 2.2));
}
