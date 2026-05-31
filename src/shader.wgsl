// Vertex shader

struct RectUniform {
    position: vec2<f32>,
    screen_size: vec2<f32>,
    size: vec2<f32>,
    camera_position: vec2<f32>,
    color: vec4<f32>,
    rotation: f32,
    _padding: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> rect: RectUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = rect.color;

    let local_pos = model.position.xy * rect.size;

    let center = rect.size * 0.5;
    let centered = local_pos - center;

    let c = cos(rect.rotation);
    let s = sin(rect.rotation);

    let rotated = vec2<f32>(
        centered.x * c - centered.y * s,
        centered.x * s + centered.y * c,
    );

    let rotated_local = rotated + center;

    let world_pos = rotated_local + rect.position;
    let screen_pos = vec2<f32>(world_pos.x - rect.camera_position.x, world_pos.y);

    let clip_x = (screen_pos.x / rect.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (screen_pos.y / rect.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(
        clip_x,
        clip_y,
        model.position.z,
        1.0
    );

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
