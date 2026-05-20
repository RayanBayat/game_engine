// Vertex shader

struct RectUniform {
    position: vec2<f32>,
    screen_size: vec2<f32>,
    size: vec2<f32>,
    _padding: vec2<f32>,
    color: vec4<f32>,
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
    let screen_pos = local_pos + rect.position;

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
