#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::forward_io::VertexOutput

const MAX_STEPS: u32 = 100;
const MAX_DISTANCE: f32 = 1000;
const EPSILON: f32 = 0.1;

const MODEL_COLOR: vec3<f32> = vec3(0.9, 0.1, 0.1);

// Hemi lighting
const SKY_COLOR: vec3<f32> = vec3(0.0, 0.3, 0.6);
const GROUND_COLOR: vec3<f32> = vec3(0.6, 0.3, 0.2);

// Diffuse lighting
const LIGHT_DIR: vec3<f32> = vec3(0.0, 1.0, 1.0);
const LIGHT_COLOR: vec3<f32> = vec3(1.0, 1.0, 0.9);

// PHONG specular
const SPECULAR_INTENSITY: f32 = 32.0;

struct SimpleMaterial {
    color: vec4<f32>
};

@group(2) @binding(0)
var<uniform> material: SimpleMaterial;

// SDF of a sphere of radius 0.5 centered at the origin
fn sdSphere(p: vec3<f32>) -> f32 {
    return length(p) - 100;
}

fn sdBox(p: vec3<f32>, b: vec3<f32> ) -> f32 {
  let q = abs(p) - b;
  return length(max(q, vec3(0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}

fn sceneSdf(p: vec3<f32>) -> f32 {
    return sdBox(p, vec3<f32>(50.0));
}

/// Use the gradient to estimate t surface normal of a point p on the 
/// surface
fn estimateNormal(p: vec3<f32>) -> vec3<f32> {
    let dx = sceneSdf(p + vec3<f32>( EPSILON, 0.0, 0.0)) - sceneSdf(p - vec3<f32>( EPSILON, 0.0, 0.0));
    let dy = sceneSdf(p + vec3<f32>(0.0,  EPSILON, 0.0)) - sceneSdf(p - vec3<f32>(0.0,  EPSILON, 0.0));
    let dz = sceneSdf(p + vec3<f32>(0.0, 0.0,  EPSILON)) - sceneSdf(p - vec3<f32>(0.0, 0.0,  EPSILON));
    return normalize(vec3<f32>(dx, dy, dz));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let origin = vec3<f32>(in.world_position.xy, 1.0);

    // Move -Z
    let dir = vec3<f32>(0.0, 0.0, -1.0);

    // Accumulated ray distance
    var t: f32 = 0.0;

    var color: vec3<f32> = vec3<f32>(0.0);

    for (var i: u32 = 0u; i < MAX_STEPS; i = i + 1u) {
        // evaluate a point along the ray
        let p = origin + dir * t;

        // calculate min distance to a surface using SDF
        let d = sceneSdf(p);

        // If we're within some threshold of a surface render it
        if (d < 0.001) {
            let normal = estimateNormal(p);

            // Ambient Light
            let ambient = vec3(0.5);

            // Hemi lighting
            let hemiMix = remap(normal.z, -1.0, 1.0, 0.0, 1.0);
            let hemi = mix(GROUND_COLOR, SKY_COLOR, hemiMix);

            // Diffuse lighting
            let lightDir = normalize(LIGHT_DIR);
            let dp = max(dot(lightDir, normal), 0.0);
            let diffuse = dp * LIGHT_COLOR;

            // Phong specular
            let r = reflect(-lightDir, normal);
            let phongValue = max(dot(dir, r), 0.0);
            let specular = pow(phongValue, SPECULAR_INTENSITY);


            let lighting = ambient * 0.1 + hemi * 0.1 + diffuse * 1.0 + specular * 0.0;

            let color = lighting * MODEL_COLOR;

            return vec4<f32>(toSRGB(color), 0.0); 
        }

        // We can always move at least d away for the next iter (sphere tracing)
        t = t + d;

        // If we get too far away we give up
        if (t > MAX_DISTANCE) {
            break;
        }
    }

    return vec4<f32>(toSRGB(color), 1.0);
}

fn remap(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (x - in_min) / (in_max - in_min);
    return t * (out_max - out_min) + out_min;
}

fn toSRGB(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(1.0 / 2.2));
}
