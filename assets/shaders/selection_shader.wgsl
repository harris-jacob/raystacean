#import bevy_pbr::forward_io::VertexOutput

#import "./shaders/sdf.wgsl"::{sd_sphere, sd_box, min_sdf, max_sdf, SdfResult}

const MAX_STEPS: i32 = 100;
const HIT_THRESHOLD: f32 = 1;
const MAX_DISTANCE: f32 = 100.0;

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
var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(3)
var<storage, read> primitives: array<GpuPrimative>;
@group(2) @binding(4)
var<storage, read> bvh_nodes: array<GpuBvhNode>;
@group(2) @binding(5)
var<storage, read> bvh_prim_indices: array<u32>;
@group(2) @binding(6)
var<storage, read_write> selection: array<f32>;

fn map(p: vec3<f32>) -> SdfResult {
    var sdf = SdfResult(100.0, BLACK);

    var stack: array<i32, 64>;
    var sp = 0;
    stack[sp] = 0;
    sp += 1;

//    while (sp > 0) {
//        sp -= 1;
//        let node_index = stack[sp];
//        let node = bvh_nodes[node_index];
//
//        // optional early-out culling
//        let aabb_dist = distance_to_aabb(p, node.bounds_min, node.bounds_max);
//        if (aabb_dist > sdf.dist) { continue; }
//
//        // Leaf node
//        if (node.left_index == -1) {
//            for (var i = 0u; i < node.prim_count; i = i + 1u) {
//                let prim_index = bvh_prim_indices[node.first_prim + i];
//                if (prim_index >= arrayLength(&primitives)) { continue; }
//
//                let prim = primitives[prim_index];
//                let b = sd_box(p - prim.position, prim.scale, prim.rounding, prim.color);
//
//                sdf = min_sdf(b, sdf);
//            }
//        } else {
//            stack[sp] = node.left_index;
//            sp += 1;
//            stack[sp] = node.right_index;
//            sp += 1;
//        }
//    }

    return sdf;
}


fn ray_march(camera_origin: vec3<f32>, camera_dir: vec3<f32>) -> vec3<f32> {
    var dist = 0.0;


    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = camera_origin + dist * camera_dir;
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


    return vec3<f32>(1.0);
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

    if distance(ndc.xy, cursor_position) < 0.001 {
        selection[0] = result.x;
        selection[1] = result.y;
        selection[2] = result.z;
    }

    return vec4<f32>(result, 1.0);

}

fn distance_to_aabb(p: vec3<f32>, bmin: vec3<f32>, bmax: vec3<f32>) -> f32 {
    let d = max(bmin - p, p - bmax);
    // clamp negative components to zero
    let outside = max(d, vec3<f32>(0.0));
    return length(outside);
}
