struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_position: vec4<f32>,
};

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) fog: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> light: LightUniform;

struct FogUniform {
    color: vec4<f32>,
    params: vec4<f32>, // x = density
};

@group(0) @binding(2)
var<uniform> fog: FogUniform;

@group(1) @binding(0)
var texture_map: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(
    @location(0) in_position: vec3<f32>,
    @location(1) in_normal: vec3<f32>,
    @location(2) in_uv: vec2<f32>,
    @location(3) instance_translation: vec3<f32>,
    @location(4) instance_scale: f32,
    @location(5) instance_color: vec3<f32>,
) -> VertexOut {
    var out: VertexOut;
    let world_pos = instance_translation + in_position * instance_scale;
    out.position = camera.view_proj * vec4(world_pos, 1.0);
    out.color = instance_color;
    out.normal = normalize(in_normal);
    out.uv = in_uv;
    let eye = camera.eye_position.xyz;
    let distance_from_camera = length(world_pos - eye);
    let fog_density = fog.params.x;
    out.fog = clamp(1.0 - exp(-fog_density * distance_from_camera), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let ambient = vec3<f32>(0.1, 0.1, 0.1);
    let light_dir = normalize(-light.direction.xyz);
    let n = normalize(in.normal);
    let diffuse_strength = max(dot(n, light_dir), 0.0);
    let diffuse_color = light.color.xyz * light.color.w * diffuse_strength;
    let lighting = ambient + diffuse_color;
    let tex_color = textureSample(texture_map, texture_sampler, in.uv);
    let base = tex_color.xyz * in.color;
    let shaded = base * lighting;
    let fogged = mix(shaded, fog.color.xyz, in.fog);
    return vec4(fogged, tex_color.w);
}
