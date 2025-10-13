const BLUE: vec3<f32> = vec3(0.0, 0.0, 1.0);

struct SdfResult {
    dist: f32,
    color: vec3<f32>,
}

fn sd_sphere(p: vec3<f32>, r: f32) -> SdfResult {
    let d = length(p) - r;
    return SdfResult(d, BLUE);
}

fn sd_box(p: vec3<f32>, b: vec3<f32>, r: f32, color: vec3<f32>) -> SdfResult {
  let q = abs(p) - b + r;
  let d = length(max(q, vec3(0.0))) + min(max(q.x,max(q.y,q.z)), 0.0) - r;
  return SdfResult(d, color);
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
