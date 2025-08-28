#import bevy_pbr::forward_io::VertexOutput

const MAX_STEPS: i32 = 100;
const HIT_THRESHOLD: f32 = 0.01;
const MAX_DISTANCE: f32 = 1000.0;

const RED: vec3<f32> = vec3(1.0, 0.0, 0.0);
const BLUE: vec3<f32> = vec3(0.0, 0.0, 1.0);
const WHITE: vec3<f32> = vec3(1.0, 1.0, 1.0);
const BLACK: vec3<f32> = vec3(0.0, 0.0, 0.0);

struct GpuBox {
    position: vec3<f32>,
    size: f32, 
    color: vec3<f32>,
    selected: u32
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

fn sd_sphere(p: vec3<f32>, r: f32) -> SdfResult {
    let d = length(p) - r;
    return SdfResult(d, BLUE);
}

fn sd_box(p: vec3<f32>, b: vec3<f32>, color: vec3<f32>) -> SdfResult {
  let q = abs(p) - b;
  let d = length(max(q, vec3(0.0))) + min(max(q.x,max(q.y,q.z)), 0.0);
  return SdfResult(d, color);
}

fn sd_box_frame(in: vec3<f32>, b: vec3<f32>, e: f32, color: vec3<f32>) -> SdfResult {
    let p = abs(in)-b;
    let q = abs(p+e)-e;
  
  return SdfResult(min(min(
      length(max(vec3(p.x,q.y,q.z), vec3(0.0)))+min(max(p.x,max(q.y,q.z)), 0.0),
      length(max(vec3(q.x,p.y,q.z), vec3(0.0)))+min(max(q.x, max(p.y,q.z)), 0.0)),
      length(max(vec3(q.x,q.y,p.z), vec3(0.0)))+min(max(q.x,max(q.y,p.z)), 0.0)
      ), color);
}

fn sd_ground(p: vec3<f32>) -> SdfResult {
  return SdfResult(-p.y, WHITE);
}

struct SdfResult {
    dist: f32,
    color: vec3<f32>,
}

fn map(p: vec3<f32>) -> SdfResult {
    var sdf =  sd_ground(p);

    for (var i = 0u; i < arrayLength(&boxes); i++) {
        let box = boxes[i];

        if(box.selected == 1) {
            let outline = sd_box_frame(p - box.position, vec3<f32>(box.size + 0.05), 0.02, RED);
            sdf = min_sdf(sdf, outline);
        }

        let b = sd_box(p - box.position, vec3<f32>(box.size), box.color);

        sdf = min_sdf(sdf, b);
    }

    return sdf;
}

fn op_subtraction(s1: SdfResult, s2: SdfResult) -> SdfResult {
    let inverted = SdfResult(-s1.dist, s1.color);

    return max_sdf(inverted, s2);
}

fn min_sdf(s1: SdfResult, s2: SdfResult) -> SdfResult {
    if (s1.dist < s2.dist) {
        return s1;
    };

    return s2;
}


fn max_sdf(s1: SdfResult, s2: SdfResult) -> SdfResult {
    if (s1.dist > s2.dist) {
        return s1;
    };

    return s2;
}

fn ray_march(cameraOrigin: vec3<f32>, cameraDir: vec3<f32>) -> vec3<f32> {
    var dist = 0.0;

    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = cameraOrigin + dist * cameraDir;
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

    let ray_dir_view = normalize(vec3<f32>(pixel_coords * 2.0 / aspect_ratio.y, 1.0));

    // Transform into world space
    let ray_origin = (camera_transform * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let ray_dir = normalize((camera_transform * vec4<f32>(ray_dir_view, 0.0)).xyz);

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
