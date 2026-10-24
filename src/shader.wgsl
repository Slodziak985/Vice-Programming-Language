// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    // Map screen coordinates to clip space. We expect positions to be pre-mapped to [-1, 1] clip space in Rust,
    // or we can pass them as screen coords and map them in WGSL. For simplicity, let's pass them as screen coords
    // and use a push constant or uniform to provide screen size.
    // However, it's easiest if Rust passes the normalized device coordinates (NDC) directly.
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
