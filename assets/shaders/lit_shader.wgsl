#import bevy_pbr::forward_io::VertexOutput
#import "./shaders/sdf.wgsl"::{sd_sphere, sd_box, min_sdf, max_sdf, SdfResult}

const MAX_STEPS: i32 = 100;
const HIT_THRESHOLD: f32 = 0.001;
const MAX_DISTANCE: f32 = 200.0;

const WORLD_BOUNDS: f32 = 128.0;
const CHUNK_SIZE: f32 = 16.0;
const RESOLUTION: u32 = 32;
const RED: vec3<f32> = vec3(1.0, 0.0, 0.0);

@group(2) @binding(0)
var<uniform> view_to_world: mat4x4<f32>;
@group(2) @binding(1)
var<uniform> clip_to_view: mat4x4<f32>;
@group(2) @binding(2)
var voxel_texture: texture_3d<f32>; 
@group(2) @binding(3)
var voxel_sampler: sampler;

const WORLD_SIZE: f32 = 256.0;

fn map(p: vec3<f32>) -> vec4<f32> {
    let tex_coord = (p + WORLD_SIZE * 0.5) / WORLD_SIZE;

    return textureSample(voxel_texture, voxel_sampler, tex_coord);
}


// Lighting method based on Inigo Quilez' raymarching - primatives demo
// https://www.shadertoy.com/view/Xds3zN
fn ray_march(camera_origin: vec3<f32>, camera_dir: vec3<f32>) -> vec3<f32> {
    var dist = 0.0;


    for (var i = 0; i < MAX_STEPS; i++) {
        var pos = camera_origin + dist * camera_dir;
        let result = map(pos);

        // Hit something
        if(result.a < HIT_THRESHOLD) {

            let lit_color = calc_lighting(pos, result.rgb, camera_dir);
            
            return lit_color;
        }

        dist = dist + result.a;

        if(result.a > MAX_DISTANCE) {
            break;
        }
    }


    // Sky color
    return sky_color(camera_dir);
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let t = clamp(0.5 + 0.5 * rd.y, 0.0, 1.0);
    let horizon = vec3<f32>(0.8, 0.9, 1.0);
    let zenith  = vec3<f32>(0.4, 0.6, 0.9);
    return mix(horizon, zenith, t);
}

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     let slice_z = 0.47; // midpoint of the 3D texture
//     let uv = in.uv;
//     let texcoord = vec3<f32>(uv, slice_z);
//     let d = textureSample(voxel_texture, voxel_sampler, texcoord).r;
// 
//     // Map distances to colors for visibility
//     let color = vec3<f32>(
//         clamp(1.0 - abs(d) * 10.0, 0.0, 1.0),
//         step(0.0, d),
//         step(d, 0.0)
//     );
//     return vec4<f32>(color, 1.0);
//}
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
 
     return vec4<f32>(result, 1.0);
 
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
    for (var i: i32 = 0; i < 32; i = i + 1) {
        let h = map(ro + rd * t).a;
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
    let dx = map(p + vec3<f32>(e,0,0)).a - map(p - vec3<f32>(e,0,0)).a;
    let dy = map(p + vec3<f32>(0,e,0)).a - map(p - vec3<f32>(0,e,0)).a;
    let dz = map(p + vec3<f32>(0,0,e)).a - map(p - vec3<f32>(0,0,e)).a;
    return normalize(vec3<f32>(dx, dy, dz));
}
