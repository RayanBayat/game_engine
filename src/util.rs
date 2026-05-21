pub fn normalize(v: [f32; 2]) -> [f32; 2] {
    let length = (v[0].powi(2) + v[1].powi(2)).sqrt();
    if length == 0.0 {
        [0.0, 0.0]
    } else {
        [v[0] / length, v[1] / length]
    }
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}