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
