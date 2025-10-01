#import bevy_pbr::forward_io::VertexOutput

const MAX_STEPS: i32 = 200;
const HIT_THRESHOLD: f32 = 0.001;
const MAX_DISTANCE: f32 = 500.0;

const RED: vec3<f32> = vec3(1.0, 0.0, 0.0);
const BLUE: vec3<f32> = vec3(0.0, 0.0, 1.0);
const WHITE: vec3<f32> = vec3(1.0, 1.0, 1.0);
const BLACK: vec3<f32> = vec3(0.0, 0.0, 0.0);

struct GpuPrimative {
    position: vec3<f32>,
    scale: vec3<f32>, 
    color: vec3<f32>,
    rounding: f32,
    logical_color: vec3<f32>,
    selected: f32,
}

struct GpuOp {
    kind: u32,
    left: u32,
    right: u32,
    primative_index: u32,
}

@group(2) @binding(0)
var<uniform> aspect_ratio: vec2<f32>;
@group(2) @binding(1)
var<uniform> view_to_world: mat4x4<f32>;
@group(2) @binding(2)
var<uniform> clip_to_view: mat4x4<f32>;
@group(2) @binding(3)
var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(4)
var<uniform> is_color_picking: u32;
@group(2) @binding(5)
var<storage, read> primatives: array<GpuPrimative>;
@group(2) @binding(6)
var<storage, read> operations: array<GpuOp>;
@group(2) @binding(7)
var<storage, read> op_roots: array<u32>;
@group(2) @binding(8)
var<storage, read_write> selection: array<f32>;


var<private> results: array<SdfResult, 10>;

fn sd_sphere(p: vec3<f32>, r: f32) -> SdfResult {
    let d = length(p) - r;
    return SdfResult(d, BLUE);
}

fn sd_box(p: vec3<f32>, b: vec3<f32>, r: f32, color: vec3<f32>) -> SdfResult {
  let q = abs(p) - b + r;
  let d = length(max(q, vec3(0.0))) + min(max(q.x,max(q.y,q.z)), 0.0) - r;
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
  return SdfResult(p.y, grid_color(p));
}

fn grid_color(pos: vec3<f32>) -> vec3<f32> {
    let minor_scale = 2.5;
    let major_scale = 0.5;
    let line_thickness = 0.01;
    
    let p_minor = pos.xz * minor_scale;
    let gx = abs(fract(p_minor.x) - 0.5);
    let gz = abs(fract(p_minor.y) - 0.5);
    let line_minor = f32(gx < line_thickness || gz < line_thickness);

    let p_major = pos.xz * major_scale;
    let mx = abs(fract(p_major.x) - 0.5);
    let mz = abs(fract(p_major.y) - 0.5);
    let line_major = f32(mx < line_thickness || mz < line_thickness);

    let base_col   = vec3<f32>(0.95, 0.97, 1.0);
    let minor_col  = vec3<f32>(0.4, 0.6, 0.9);
    let major_col  = vec3<f32>(0.1, 0.3, 0.6);

    var col = base_col;
    if (line_minor > 0.5) { col = minor_col; }
    if (line_major > 0.5) { col = major_col; }

    return col;
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let t = clamp(0.5 + 0.5 * rd.y, 0.0, 1.0);
    let horizon = vec3<f32>(0.8, 0.9, 1.0);
    let zenith  = vec3<f32>(0.4, 0.6, 0.9);
    return mix(horizon, zenith, t);
}

struct SdfResult {
    dist: f32,
    color: vec3<f32>,
}

fn map(p: vec3<f32>) -> SdfResult {
    if (is_color_picking != 0) {
        return map2(p);
    } else {
        return map3(p);
    }
}

fn map2(p: vec3<f32>) -> SdfResult {
    var sdf = SdfResult(100.0, BLACK);

    for (var i = 0u; i < arrayLength(&primatives); i++) {
        let box = primatives[i];

        let color = color_from_box(box);
        let b = sd_box(p - box.position, box.scale, box.rounding, color);

        sdf = min_sdf(sdf, b);
    }

    if(is_color_picking == 0) {
        sdf =  min_sdf(sd_ground(p), sdf);
    }

    return sdf;
}

fn map3(p: vec3<f32>) -> SdfResult {

    for (var i: u32 = 0u; i < arrayLength(&operations); i = i + 1u) {
        let node = operations[i];

        if (node.kind == 0u) {
            let prim = primatives[node.primative_index];
            let color = color_from_box(prim);
            results[i] = sd_box(p - prim.position, prim.scale, prim.rounding, color);

        } else if (node.kind == 1u) { // union
            results[i].dist = op_smooth_union(results[node.left].dist, results[node.right].dist, 0.5);
            // TODO: eventually provide a color override for ops
            results[i].color = results[node.left].color;
        }
    }

    var sdf = SdfResult(100.0, BLACK);

    for (var i = 0u;  i < arrayLength(&op_roots); i = i + 1u) {
        let idx = op_roots[i];
        let operation_sdf = results[idx];

        sdf = min_sdf(sdf, operation_sdf);
    }

    if(is_color_picking == 0) {
        sdf =  min_sdf(sd_ground(p), sdf);
    }

    return sdf;
}

fn color_from_box(box: GpuPrimative) -> vec3<f32> {
    if(is_color_picking != 0) {
        return box.logical_color;
    } else {
        return box.color;
    }
}

fn op_subtraction(s1: SdfResult, s2: SdfResult) -> SdfResult {
    let inverted = SdfResult(-s1.dist, s1.color);

    return max_sdf(inverted, s2);
}

fn op_smooth_union(s1: f32, s2: f32, b: f32) -> f32 {
    let k = b * 4.0;
    let h = max(k - abs(s1 - s2), 0.0);

    return min(s1, s2) - h*h*0.25/k;
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

// Lighting method based on Inigo Quilez' raymarching - primatives demo
// https://www.shadertoy.com/view/Xds3zN
fn ray_march(camera_origin: vec3<f32>, camera_dir: vec3<f32>) -> vec3<f32> {
    var dist = 0.0;


    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = camera_origin + dist * camera_dir;
        let result = map(pos);

        // Hit something
        if(result.dist < HIT_THRESHOLD) {

            let lit_color = calc_lighting(pos, result.color, camera_dir);
            
            return lit_color;
        }

        dist = dist + result.dist;

        if(result.dist > MAX_DISTANCE) {
            break;
        }
    }


    // Sky color
    return sky_color(camera_dir);
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. UV → NDC
    let ndc = vec4<f32>(
        in.uv * 2.0 - vec2<f32>(1.0, 1.0),
        -1.0,
        1.0
    );

    // 2. NDC → view space
    let view_pos_h = clip_to_view * ndc;
    let view_pos   = view_pos_h.xyz / view_pos_h.w;

    // 3. Ray in view space
    let ray_origin_view = vec3<f32>(0.0, 0.0, 0.0);
    let ray_dir_view = normalize(view_pos);

    // 4. Transform into world space
    let ray_origin_world = (view_to_world * vec4<f32>(ray_origin_view, 1.0)).xyz;
    let ray_dir_world    = normalize((view_to_world * vec4<f32>(ray_dir_view, 0.0)).xyz);

    // 5. March in world space
    let result = ray_march(ray_origin_world, ray_dir_world);

    if is_color_picking != 0 && distance(ndc.xy, cursor_position) < 0.001 {
        selection[0] = result.x;
        selection[1] = result.y;
        selection[2] = result.z;
    }


    return vec4<f32>(result, 1.0);

}

fn calc_lighting(pos: vec3<f32>, in: vec3<f32>, camera_dir: vec3<f32>) -> vec3<f32> {
    // Don't bother with lighting for color picking pass
    if (is_color_picking != 0) {
        return in;
    }

    let sun_dir = normalize(vec3<f32>(-0.5, 0.4, -0.6));
    let half_dir = normalize(sun_dir - camera_dir);

    let normal = calc_normal(pos);
    let reflected = reflect(camera_dir, normal);


    var color = vec3<f32>(0.0);  


    {
        // diffuse
        var diff = clamp(dot(normal, sun_dir), 0.0, 1.0);
        diff *= soft_shadow(pos, sun_dir, 0.02, 2.5);
        // Blinn-phong Specular
        var spec = pow(max(dot(normal, half_dir), 0.0), 16.0);
        spec *= diff;
        // Fresnel
        spec *= 0.04+0.96*pow(clamp(1.0-dot(half_dir, sun_dir), 0.0, 1.0), 5.0);

        let diffuse_color = in * 1.8*diff*vec3<f32>(1.30, 1.00, 0.70);
        let specular_color = 5.00*spec*vec3<f32>(1.30, 1.0, 0.7);
        color += diffuse_color+specular_color;
    }

        // Sky light;
    {
        // diff
        let diff = sqrt(clamp(0.5 + 0.5*normal.y, 0.0, 1.0));
        var spec = smoothstep(-0.2, 0.2, reflected.y);
        spec *= diff;
        // Fresnel
        spec *= 0.04+0.96*pow(clamp(1.0+dot(normal, camera_dir), 0.0, 1.0), 5.0);
        spec *= soft_shadow(pos, reflected, 0.02, 2.5);

        color += in * 0.4 * diff * vec3<f32>(0.4, 0.6, 1.15);
        color += 1.00*spec*vec3<f32>(0.4, 0.6, 1.30);
    }

    return color;
}

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>, min_dist: f32, max_dist: f32) -> f32 {
    var t: f32 = min_dist;
    var res: f32 = 1.0;
    for (var i: i32 = 0; i < 32; i = i + 1) {
        let h = map(ro + rd * t).dist;
        if (h < 0.001) {
            return 0.0;
        }
        res = min(res, 16.0 * h / t);
        t = t + h;
        if (t > max_dist) { break; }
    }
    return clamp(res, 0.0, 1.0);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e: f32 = 0.001;
    let dx = map(p + vec3<f32>(e,0,0)).dist - map(p - vec3<f32>(e,0,0)).dist;
    let dy = map(p + vec3<f32>(0,e,0)).dist - map(p - vec3<f32>(0,e,0)).dist;
    let dz = map(p + vec3<f32>(0,0,e)).dist - map(p - vec3<f32>(0,0,e)).dist;
    return normalize(vec3<f32>(dx, dy, dz));
}

fn remap(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (x - in_min) / (in_max - in_min);
    return t * (out_max - out_min) + out_min;
}

fn toSRGB(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 / 2.2));
}
