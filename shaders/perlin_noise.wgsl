@group(0) @binding(0)
var<uniform> u_Time: f32;

fn hash(p: vec2<f32>) -> f32 {
    let p3 = fract(p * vec2<f32>(0.1031, 0.1030));
    p3 += dot(p3, p3.yx + 33.33);
    return fract((p3.x + p3.y) * p3.x);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    let u = f * f * (3.0 - 2.0 * f);
    return mix(a, b, u.x) + (c - a) * u.y * (d - b) * u.x * u.y;
}

@fragment
fn main(@location(0) v_Uv: vec2<f32>) -> @location(0) vec4<f32> {
    let uv = v_Uv * 10.0 + vec2<f32>(u_Time * 0.1, 0.0);
    let n = noise(uv);
    return vec4<f32>(vec3<f32>(n), 1.0);
}
