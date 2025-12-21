struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye_position: vec4<f32>,
};

const MAX_POINT_LIGHTS: u32 = 8u;

struct LightUniform {
    directional_direction: vec4<f32>,
    directional_color: vec4<f32>,
    point_positions: array<vec4<f32>, MAX_POINT_LIGHTS>,
    point_colors: array<vec4<f32>, MAX_POINT_LIGHTS>,
    point_count: vec4<u32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) fog: f32,
    @location(4) world_pos: vec3<f32>,
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

fn quat_mul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    );
}

fn quat_conjugate(q: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(-q.xyz, q.w);
}

fn normalize_quat(q: vec4<f32>) -> vec4<f32> {
    let len_sq = dot(q, q);
    if (len_sq <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return q / sqrt(len_sq);
}

fn rotate_vector(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let qv = vec4<f32>(v, 0.0);
    let rotated = quat_mul(quat_mul(q, qv), quat_conjugate(q));
    return rotated.xyz;
}

@vertex
fn vs_main(
    @location(0) in_position: vec3<f32>,
    @location(1) in_normal: vec3<f32>,
    @location(2) in_uv: vec2<f32>,
    @location(3) instance_translation: vec3<f32>,
    @location(4) instance_scale: f32,
    @location(5) instance_color: vec3<f32>,
    @location(6) instance_rotation: vec4<f32>,
) -> VertexOut {
    var out: VertexOut;
    let rotation = normalize_quat(instance_rotation);
    let local_pos = rotate_vector(in_position * instance_scale, rotation);
    let world_pos = instance_translation + local_pos;
    out.position = camera.view_proj * vec4(world_pos, 1.0);
    out.color = instance_color;
    out.normal = normalize(rotate_vector(in_normal, rotation));
    out.uv = in_uv;
    out.world_pos = world_pos;
    let eye = camera.eye_position.xyz;
    let distance_from_camera = length(world_pos - eye);
    let fog_density = fog.params.x;
    out.fog = clamp(1.0 - exp(-fog_density * distance_from_camera), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let ambient = vec3<f32>(0.1, 0.1, 0.1);
    let n = normalize(in.normal);
    var lighting = ambient;

    if (light.directional_color.w > 0.0) {
        let light_dir = normalize(-light.directional_direction.xyz);
        let diffuse_strength = max(dot(n, light_dir), 0.0);
        let diffuse_color = light.directional_color.xyz * light.directional_color.w * diffuse_strength;
        lighting = lighting + diffuse_color;
    }

    let point_count = light.point_count.x;
    for (var i: u32 = 0u; i < point_count && i < MAX_POINT_LIGHTS; i = i + 1u) {
        let point_data = light.point_positions[i];
        let point_color = light.point_colors[i];
        let to_light = point_data.xyz - in.world_pos;
        let dist = length(to_light);
        if (point_color.w > 0.0 && point_data.w > 0.0) {
            let dir = normalize(to_light);
            let radius = point_data.w;
            let attenuation = max(1.0 - dist / radius, 0.0);
            let diffuse_strength = max(dot(n, dir), 0.0);
            let diffuse_color = point_color.xyz * point_color.w * diffuse_strength * attenuation * attenuation;
            lighting = lighting + diffuse_color;
        }
    }

    let tex_color = textureSample(texture_map, texture_sampler, in.uv);
    if (tex_color.a < 0.5) {
        discard;
    }
    let base = tex_color.xyz * in.color;
    let shaded = base * lighting;
    let fogged = mix(shaded, fog.color.xyz, in.fog);
    return vec4(fogged, tex_color.w);
}
