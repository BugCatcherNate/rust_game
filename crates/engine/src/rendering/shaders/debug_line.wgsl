struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LineVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) in_position: vec3<f32>, @location(1) in_color: vec3<f32>) -> LineVertexOut {
    var out: LineVertexOut;
    out.position = camera.view_proj * vec4(in_position, 1.0);
    out.color = in_color;
    return out;
}

@fragment
fn fs_main(in: LineVertexOut) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.0);
}
