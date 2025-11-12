#import bevy_pbr::forward_io::VertexOutput

#import "./shaders/sdf.wgsl"::{sd_sphere, sd_box, min_sdf, max_sdf, SdfResult}

const MAX_STEPS: i32 = 64;
const HIT_THRESHOLD: f32 = 0.01;
const MAX_DISTANCE: f32 = 100.0;

const BLACK: vec3<f32> = vec3(0.0, 0.0, 0.0);

struct GpuPrimative {
    position: vec3<f32>,
    is_subtract: u32,
    scale: vec3<f32>, 
    blend: f32,
    color: vec3<f32>,
    rounding: f32,
    logical_color: vec3<f32>,
}

struct GpuBvhNode {
    bounds_min: vec3<f32>,
    _pad0: u32,
    bounds_max: vec3<f32>,
    _pad1: u32,
    left_index: i32,
    right_index: i32,
    first_prim: u32,
    prim_count: u32,
}

@group(2) @binding(0)
var<uniform> view_to_world: mat4x4<f32>;
@group(2) @binding(1)
var<uniform> clip_to_view: mat4x4<f32>;
@group(2) @binding(2)
var<storage, read> primitives: array<GpuPrimative>;
@group(2) @binding(3)
var<storage, read> bvh_nodes: array<GpuBvhNode>;
@group(2) @binding(4)
var<storage, read> bvh_prim_indices: array<u32>;

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let t = clamp(0.5 + 0.5 * rd.y, 0.0, 1.0);
    let horizon = vec3<f32>(0.8, 0.9, 1.0);
    let zenith  = vec3<f32>(0.4, 0.6, 0.9);
    return mix(horizon, zenith, t);
}

// fn map(p: vec3<f32>) -> SdfResult {
//    return SdfResult(100.0, BLACK);
// }

struct MapResult {
    sdf: SdfResult,
    prims_evaluated: f32,
}

fn map(p: vec3<f32>) -> MapResult {
     var sdf = SdfResult(100.0, BLACK);
     var prims_evaluated = 0.0;
 
     var stack: array<i32, 20>;
     var sp = 0;
     stack[sp] = 0;
     sp += 1;
 
     while (sp > 0) {
         sp -= 1;
         let node_index = stack[sp];
         let node = bvh_nodes[node_index];
 
         // optional early-out culling
         let aabb_dist = distance_to_aabb(p, node.bounds_min, node.bounds_max);
         if (aabb_dist > sdf.dist) { continue; }
 
         // Leaf node
         if (node.left_index == -1) {
             for (var i = 0u; i < node.prim_count; i = i + 1u) {
                 let prim_index = bvh_prim_indices[node.first_prim + i];
                 if (prim_index >= arrayLength(&primitives)) { continue; }
 
                 let prim = primitives[prim_index];
                 let b = sd_box(p - prim.position, prim.scale, prim.rounding, prim.color);
                 prims_evaluated += 1.0;
 
                 if (prim.is_subtract == 1u) {
                     sdf.dist = op_smooth_subtract(b.dist, sdf.dist, prim.blend);
                 } else {
                     sdf = sd_smooth_union(sdf, b, prim.blend);
                 }
             }
         } else {
             stack[sp] = node.left_index;
             sp += 1;
             stack[sp] = node.right_index;
             sp += 1;
         }
     }
 
     // sdf = min_sdf(sdf, sd_ground(p));
     return MapResult(sdf, prims_evaluated);
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

// Lighting method based on Inigo Quilez' raymarching - primatives demo
// https://www.shadertoy.com/view/Xds3zN
fn ray_march(camera_origin: vec3<f32>, camera_dir: vec3<f32>) -> f32 {
    var dist = 0.0;
    var prims = 0.0;


    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = camera_origin + dist * camera_dir;
        let result = map(pos);

        // Hit something
        if(result.sdf.dist < HIT_THRESHOLD) {

            let lit_color = calc_lighting(pos, result.sdf.color, camera_dir);
            
            return result.prims_evaluated;
        }

        dist = dist + result.sdf.dist;

        if(dist > MAX_DISTANCE) {
            return result.prims_evaluated;
        }

        prims = result.prims_evaluated;
    }


    // Sky color
    return prims;
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
    let t = clamp(result / f32(arrayLength(&primitives)), 0.0, 1.0);
    let heat_color = vec3<f32>(t, 0.0, 1.0 - t);

    return vec4<f32>(heat_color, 1.0);

}

fn calc_lighting(pos: vec3<f32>, in: vec3<f32>, camera_dir: vec3<f32>) -> vec3<f32> {
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
    for (var i: i32 = 0; i < 20; i = i + 1) {
        let h = map(ro + rd * t).sdf.dist;
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
    let dx = map(p + vec3<f32>(e,0,0)).sdf.dist - map(p - vec3<f32>(e,0,0)).sdf.dist;
    let dy = map(p + vec3<f32>(0,e,0)).sdf.dist - map(p - vec3<f32>(0,e,0)).sdf.dist;
    let dz = map(p + vec3<f32>(0,0,e)).sdf.dist - map(p - vec3<f32>(0,0,e)).sdf.dist;
    return normalize(vec3<f32>(dx, dy, dz));
}


fn distance_to_aabb(p: vec3<f32>, bmin: vec3<f32>, bmax: vec3<f32>) -> f32 {
    let d = max(bmin - p, p - bmax);
    // clamp negative components to zero
    let outside = max(d, vec3<f32>(0.0));
    return length(outside);
}
