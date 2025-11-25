struct UiVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> UiVertexOut {
    var out: UiVertexOut;
    out.position = vec4<f32>(position, 1.0);
    out.uv = uv;
    return out;
}

@group(0) @binding(0)
var ui_texture: texture_2d<f32>;

@group(0) @binding(1)
var ui_sampler: sampler;

@fragment
fn fs_main(in: UiVertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(ui_texture, ui_sampler, in.uv);
    return color;
}
