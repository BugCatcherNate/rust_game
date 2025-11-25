struct Colors {
    top: vec4<f32>,
    bottom: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> colors: Colors;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    // Map clip-space Y (-1..1) to top-down UV; flip so top is 1.0
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, position.y * -0.5 + 0.5);
    return out;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t = clamp(uv.y, 0.0, 1.0);
    let color = mix(colors.bottom.xyz, colors.top.xyz, t);
    return vec4<f32>(color, 1.0);
}
