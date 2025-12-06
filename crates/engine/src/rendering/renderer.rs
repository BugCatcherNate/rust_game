use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::components::{CameraComponent, LightKind, Position};
use crate::ecs::ECS;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use log::{error, warn};
use wgpu::util::DeviceExt;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const DEFAULT_BG_TOP: [f32; 3] = [0.18, 0.26, 0.42];
const DEFAULT_BG_BOTTOM: [f32; 3] = [0.03, 0.03, 0.08];
const DEFAULT_FOG_COLOR: [f32; 3] = [0.18, 0.22, 0.28];
const DEFAULT_FOG_DENSITY: f32 = 0.45;
const UI_FONT_WIDTH: usize = 5;
const UI_FONT_HEIGHT: usize = 7;
const UI_FONT_SCALE: usize = 2;
const UI_CHAR_SPACING: usize = 1;
const UI_LINE_SPACING: usize = 2;
const UI_TEXT_PADDING_X: usize = 2;
const UI_TEXT_PADDING_Y: usize = 2;
const UI_TEXT_MARGIN_X: f32 = 16.0;
const UI_TEXT_MARGIN_Y: f32 = 20.0;

const FULLSCREEN_VERTICES: [BackgroundVertex; 6] = [
    BackgroundVertex {
        position: [-1.0, -1.0],
    },
    BackgroundVertex {
        position: [1.0, -1.0],
    },
    BackgroundVertex {
        position: [1.0, 1.0],
    },
    BackgroundVertex {
        position: [-1.0, -1.0],
    },
    BackgroundVertex {
        position: [1.0, 1.0],
    },
    BackgroundVertex {
        position: [-1.0, 1.0],
    },
];

const GLYPH_QUESTION: [u8; UI_FONT_HEIGHT] = [
    0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
];

fn glyph_bits(ch: char) -> [u8; UI_FONT_HEIGHT] {
    let c = if ch.is_ascii_lowercase() {
        ch.to_ascii_uppercase()
    } else {
        ch
    };
    match c {
        ' ' => [0; UI_FONT_HEIGHT],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '"' => [
            0b01010, 0b01010, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '#' => [
            0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b01010,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        ';' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '?' => GLYPH_QUESTION,
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '\'' => [
            0b00100, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        _ => GLYPH_QUESTION,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    eye_position: [f32; 4],
}

const MAX_POINT_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightUniform {
    directional_direction: [f32; 4],
    directional_color: [f32; 4],
    point_positions: [[f32; 4]; MAX_POINT_LIGHTS],
    point_colors: [[f32; 4]; MAX_POINT_LIGHTS],
    point_count: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FogUniform {
    color: [f32; 4],
    params: [f32; 4], // x = density
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ModelVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl ModelVertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    translation: [f32; 3],
    scale: f32,
    color: [f32; 3],
    _padding: f32,
}

impl InstanceRaw {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![3 => Float32x3, 4 => Float32, 5 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct DebugLineVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl DebugLineVertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugLineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UiVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl UiVertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ModelKey {
    model: String,
    texture: Option<String>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BackgroundVertex {
    position: [f32; 2],
}

impl BackgroundVertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BackgroundVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

struct TextureEntry {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: Arc<wgpu::BindGroup>,
    byte_size: usize,
}

#[derive(Clone, Copy)]
enum UiOverlayKind {
    Top,
    Bottom,
}

struct UiOverlay {
    text: String,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
    texture: Option<TextureEntry>,
    texture_size: (u32, u32),
}

impl UiOverlay {
    fn new() -> Self {
        Self {
            text: String::new(),
            vertex_buffer: None,
            vertex_count: 0,
            texture: None,
            texture_size: (0, 0),
        }
    }

    fn reset_geometry(&mut self) {
        self.vertex_buffer = None;
        self.vertex_count = 0;
        self.texture = None;
        self.texture_size = (0, 0);
    }
}

struct ModelEntry {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    instance_count: u32,
    texture_bind_group: Arc<wgpu::BindGroup>,
    vertex_bytes: usize,
    index_bytes: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BackgroundUniform {
    top_color: [f32; 4],
    bottom_color: [f32; 4],
}

impl BackgroundUniform {
    fn new(top: [f32; 3], bottom: [f32; 3]) -> Self {
        Self {
            top_color: [top[0], top[1], top[2], 1.0],
            bottom_color: [bottom[0], bottom[1], bottom[2], 1.0],
        }
    }
}

struct DepthTexture {
    view: wgpu::TextureView,
}

#[derive(Default)]
struct InstanceBufferPool {
    free: Vec<(usize, Vec<wgpu::Buffer>)>,
}

impl InstanceBufferPool {
    fn new() -> Self {
        Self { free: Vec::new() }
    }

    fn acquire(
        &mut self,
        device: &wgpu::Device,
        min_capacity: usize,
        stride: usize,
    ) -> (wgpu::Buffer, usize) {
        let mut best_index: Option<usize> = None;
        let mut best_capacity = usize::MAX;
        for (i, (capacity, buffers)) in self.free.iter().enumerate() {
            if *capacity >= min_capacity && !buffers.is_empty() && *capacity < best_capacity {
                best_index = Some(i);
                best_capacity = *capacity;
            }
        }

        if let Some(i) = best_index {
            let capacity = self.free[i].0;
            let buffer = self.free[i].1.pop().expect("buffer list empty");
            if self.free[i].1.is_empty() {
                self.free.swap_remove(i);
            }
            return (buffer, capacity);
        }

        let capacity = min_capacity;
        let buffer_size = (capacity * stride) as wgpu::BufferAddress;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        (buffer, capacity)
    }

    fn release(&mut self, buffer: wgpu::Buffer, capacity: usize) {
        if capacity == 0 {
            return;
        }
        if let Some((_, list)) = self.free.iter_mut().find(|(cap, _)| *cap == capacity) {
            list.push(buffer);
        } else {
            self.free.push((capacity, vec![buffer]));
        }
    }

    fn total_memory(&self, stride: usize) -> usize {
        self.free
            .iter()
            .map(|(capacity, buffers)| capacity * stride * buffers.len())
            .sum()
    }
}

fn ensure_instance_capacity_for_entry(
    device: &wgpu::Device,
    pool: &mut InstanceBufferPool,
    entry: &mut ModelEntry,
    desired: usize,
) {
    if desired == 0 {
        entry.instance_count = 0;
        return;
    }
    if entry.instance_capacity >= desired {
        return;
    }
    let capacity = desired.next_power_of_two().max(1);

    if let Some(buffer) = entry.instance_buffer.take() {
        pool.release(buffer, entry.instance_capacity);
    }

    let (buffer, actual_capacity) =
        pool.acquire(device, capacity, std::mem::size_of::<InstanceRaw>());

    entry.instance_buffer = Some(buffer);
    entry.instance_capacity = actual_capacity;
}

#[derive(Debug, Clone, Copy)]
pub struct GpuMemoryUsage {
    pub model_bytes: usize,
    pub texture_bytes: usize,
}

impl GpuMemoryUsage {
    pub fn total_bytes(&self) -> usize {
        self.model_bytes + self.texture_bytes
    }
}

#[derive(Debug, Clone)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
}

pub struct Renderer {
    _window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    fog_uniform: FogUniform,
    fog_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textures: HashMap<String, TextureEntry>,
    default_texture: TextureEntry,
    background_pipeline: wgpu::RenderPipeline,
    background_vertex_buffer: wgpu::Buffer,
    background_buffer: wgpu::Buffer,
    background_bind_group: wgpu::BindGroup,
    ui_pipeline: wgpu::RenderPipeline,
    ui_top: UiOverlay,
    ui_bottom: UiOverlay,
    clear_color: [f32; 3],
    depth_texture: DepthTexture,
    projection: Mat4,
    view_matrix: Mat4,
    models: HashMap<ModelKey, ModelEntry>,
    camera_position: [f32; 3],
    instance_pool: InstanceBufferPool,
    debug_line_pipeline: wgpu::RenderPipeline,
    debug_line_buffer: Option<wgpu::Buffer>,
    debug_line_capacity: usize,
    debug_line_vertex_count: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("wgpu device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| {
                matches!(
                    mode,
                    wgpu::PresentMode::FifoRelaxed | wgpu::PresentMode::Mailbox
                )
            })
            .unwrap_or(wgpu::PresentMode::Fifo);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let initial_eye = Vec3::new(0.0, 2.0, 5.0);
        let projection = build_projection_matrix(size);
        let view_matrix = Mat4::look_at_rh(initial_eye, Vec3::ZERO, Vec3::Y);
        let camera_uniform = CameraUniform {
            view_proj: (projection * view_matrix).to_cols_array_2d(),
            eye_position: [initial_eye.x, initial_eye.y, initial_eye.z, 1.0],
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_uniform = LightUniform {
            directional_direction: [0.0, -1.0, 0.0, 0.0],
            directional_color: [1.0, 1.0, 1.0, 1.0],
            point_positions: [[0.0; 4]; MAX_POINT_LIGHTS],
            point_colors: [[0.0; 4]; MAX_POINT_LIGHTS],
            point_count: [0, 0, 0, 0],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::bytes_of(&light_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fog_uniform = FogUniform {
            color: [
                DEFAULT_FOG_COLOR[0],
                DEFAULT_FOG_COLOR[1],
                DEFAULT_FOG_COLOR[2],
                1.0,
            ],
            params: [DEFAULT_FOG_DENSITY, 0.0, 0.0, 0.0],
        };

        let fog_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fog Buffer"),
            contents: bytemuck::bytes_of(&fog_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: fog_buffer.as_entire_binding(),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Model Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });

        let default_texture = Self::create_default_texture(
            &device,
            &queue,
            &texture_bind_group_layout,
            [255, 255, 255, 255],
        );

        let background_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Background Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let background_uniform = BackgroundUniform::new(DEFAULT_BG_TOP, DEFAULT_BG_BOTTOM);
        let background_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Uniform Buffer"),
            contents: bytemuck::bytes_of(&background_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let background_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Background Bind Group"),
            layout: &background_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: background_buffer.as_entire_binding(),
            }],
        });

        let background_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Background Vertex Buffer"),
                contents: bytemuck::cast_slice(&FULLSCREEN_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let background_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Background Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/background.wgsl").into()),
        });

        let background_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Background Pipeline Layout"),
                bind_group_layouts: &[&background_bind_group_layout],
                push_constant_ranges: &[],
            });

        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Pipeline"),
            layout: Some(&background_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &background_shader,
                entry_point: Some("vs_main"),
                buffers: &[BackgroundVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &background_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        let ui_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&ui_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: Some("vs_main"),
                buffers: &[UiVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Model Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ModelVertex::layout(), InstanceRaw::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let depth_texture = create_depth_texture(&device, &config);
        let debug_line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/debug_line.wgsl").into()),
        });
        let debug_line_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug Line Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let debug_line_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Debug Line Pipeline"),
                layout: Some(&debug_line_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &debug_line_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[DebugLineVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &debug_line_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let mut renderer = Self {
            _window: window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            background_pipeline,
            ui_pipeline,
            camera_uniform,
            camera_buffer,
            light_uniform,
            light_buffer,
            fog_uniform,
            fog_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            textures: HashMap::new(),
            default_texture,
            background_vertex_buffer,
            background_buffer,
            background_bind_group,
            ui_top: UiOverlay::new(),
            ui_bottom: UiOverlay::new(),
            clear_color: DEFAULT_BG_BOTTOM,
            depth_texture,
            projection,
            view_matrix,
            models: HashMap::new(),
            camera_position: [initial_eye.x, initial_eye.y, initial_eye.z],
            instance_pool: InstanceBufferPool::new(),
            debug_line_pipeline,
            debug_line_buffer: None,
            debug_line_capacity: 0,
            debug_line_vertex_count: 0,
        };

        renderer.set_background_colors(DEFAULT_BG_TOP, DEFAULT_BG_BOTTOM);
        renderer.set_fog(DEFAULT_FOG_COLOR, DEFAULT_FOG_DENSITY);

        renderer
    }

    pub fn total_gpu_memory(&self) -> usize {
        let model_sum: usize = self
            .models
            .values()
            .map(|entry| entry.vertex_bytes + entry.index_bytes)
            .sum();
        let texture_sum: usize = self.textures.values().map(|entry| entry.byte_size).sum();
        let pooled_instances = self
            .instance_pool
            .total_memory(std::mem::size_of::<InstanceRaw>());
        model_sum + texture_sum + pooled_instances
    }

    pub fn set_debug_lines(&mut self, lines: &[DebugLine]) {
        let vertex_count = lines.len() * 2;
        if vertex_count == 0 {
            self.debug_line_vertex_count = 0;
            return;
        }
        if self.debug_line_capacity < vertex_count {
            let capacity = vertex_count.next_power_of_two().max(2);
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Debug Line Buffer"),
                size: (capacity * std::mem::size_of::<DebugLineVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.debug_line_buffer = Some(buffer);
            self.debug_line_capacity = capacity;
        }
        let mut vertices = Vec::with_capacity(vertex_count);
        for line in lines {
            vertices.push(DebugLineVertex {
                position: line.start,
                color: line.color,
            });
            vertices.push(DebugLineVertex {
                position: line.end,
                color: line.color,
            });
        }
        if let Some(buffer) = &self.debug_line_buffer {
            self.queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(&vertices));
            self.debug_line_vertex_count = vertex_count as u32;
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = create_depth_texture(&self.device, &self.config);
            self.projection = build_projection_matrix(self.size);
            self.update_camera_uniform();
            self.rebuild_ui_vertices_for(UiOverlayKind::Top);
            self.rebuild_ui_vertices_for(UiOverlayKind::Bottom);
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn update_scene(&mut self, ecs: &ECS) {
        for entry in self.models.values_mut() {
            entry.instance_count = 0;
        }

        let mut grouped_instances: HashMap<ModelKey, Vec<InstanceRaw>> = HashMap::new();

        for archetype in &ecs.archetypes {
            for (index, position) in archetype.positions.iter().enumerate() {
                let renderable = archetype
                    .renderables
                    .as_ref()
                    .and_then(|column| column.get(index));
                let model = archetype
                    .models
                    .as_ref()
                    .and_then(|column| column.get(index));
                let texture = archetype
                    .textures
                    .as_ref()
                    .and_then(|column| column.get(index));

                if let (Some(renderable), Some(model)) = (renderable, model) {
                    let key = ModelKey {
                        model: model.asset_path.clone(),
                        texture: texture.map(|t| t.asset_path.clone()),
                    };
                    grouped_instances.entry(key).or_default().push(InstanceRaw {
                        translation: [position.x, position.y, position.z],
                        scale: renderable.size,
                        color: renderable.color,
                        _padding: 0.0,
                    });
                }
            }
        }

        let mut instance_pool = std::mem::take(&mut self.instance_pool);
        for (key, instances) in grouped_instances {
            let instance_count = instances.len();
            if instance_count == 0 {
                continue;
            }

            if !self.ensure_model_loaded(&key) {
                continue;
            }

            if let Some(model_entry) = self.models.get_mut(&key) {
                ensure_instance_capacity_for_entry(
                    &self.device,
                    &mut instance_pool,
                    model_entry,
                    instance_count,
                );
                if let Some(buffer) = &model_entry.instance_buffer {
                    self.queue
                        .write_buffer(buffer, 0, bytemuck::cast_slice(&instances));
                    model_entry.instance_count = instance_count as u32;
                }
            }
        }
        self.instance_pool = instance_pool;
    }

    fn ensure_model_loaded(&mut self, key: &ModelKey) -> bool {
        if self.models.contains_key(key) {
            return true;
        }

        match self.load_model(&key.model) {
            Ok(mut entry) => {
                entry.texture_bind_group = if let Some(path) = key.texture.as_ref() {
                    match self.ensure_texture_loaded(path) {
                        Some(texture) => Arc::clone(&texture.bind_group),
                        None => {
                            warn!(
                                "Using default texture for model '{}' due to earlier errors",
                                key.model
                            );
                            Arc::clone(&self.default_texture.bind_group)
                        }
                    }
                } else {
                    Arc::clone(&self.default_texture.bind_group)
                };
                self.models.insert(key.clone(), entry);
                true
            }
            Err(err) => {
                error!("{}", err);
                false
            }
        }
    }

    fn load_model(&self, path: &str) -> Result<ModelEntry, String> {
        let options = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };

        let (models, _) = tobj::load_obj(Path::new(path), &options)
            .map_err(|err| format!("Failed to load OBJ '{}': {}", path, err))?;

        if models.is_empty() {
            return Err(format!("OBJ '{}' contains no meshes", path));
        }

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0u32;

        for model in models {
            let mesh = model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            for i in 0..num_vertices {
                let position = [
                    mesh.positions[3 * i],
                    mesh.positions[3 * i + 1],
                    mesh.positions[3 * i + 2],
                ];
                let normal = if !mesh.normals.is_empty() {
                    [
                        mesh.normals[3 * i],
                        mesh.normals[3 * i + 1],
                        mesh.normals[3 * i + 2],
                    ]
                } else {
                    [0.0, 1.0, 0.0]
                };
                let uv = if !mesh.texcoords.is_empty() {
                    [mesh.texcoords[2 * i], mesh.texcoords[2 * i + 1]]
                } else {
                    [
                        mesh.positions[3 * i] * 0.5 + 0.5,
                        mesh.positions[3 * i + 2] * 0.5 + 0.5,
                    ]
                };
                vertices.push(ModelVertex {
                    position,
                    normal,
                    uv,
                });
            }

            indices.extend(mesh.indices.iter().map(|index| (*index) + vertex_offset));
            vertex_offset += num_vertices as u32;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let vertex_bytes = vertices.len() * std::mem::size_of::<ModelVertex>();
        let index_bytes = indices.len() * std::mem::size_of::<u32>();

        Ok(ModelEntry {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instance_buffer: None,
            instance_capacity: 0,
            instance_count: 0,
            texture_bind_group: Arc::clone(&self.default_texture.bind_group),
            vertex_bytes,
            index_bytes,
        })
    }

    fn ensure_texture_loaded(&mut self, path: &str) -> Option<&TextureEntry> {
        if !self.textures.contains_key(path) {
            match Self::load_texture_from_path(
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
                path,
            ) {
                Ok(entry) => {
                    self.textures.insert(path.to_string(), entry);
                }
                Err(err) => {
                    error!("{}", err);
                    return None;
                }
            }
        }
        self.textures.get(path)
    }

    fn load_texture_from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        path: &str,
    ) -> Result<TextureEntry, String> {
        let dyn_image = image::open(path)
            .map_err(|err| format!("Failed to load texture '{}': {}", path, err))?;
        let rgba = dyn_image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Self::create_texture_from_rgba(
            device,
            queue,
            layout,
            width,
            height,
            rgba.as_raw(),
            Some(path),
        ))
    }

    fn create_texture_from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> TextureEntry {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        }));
        TextureEntry {
            _texture: texture,
            _view: view,
            _sampler: sampler,
            bind_group,
            byte_size: data.len(),
        }
    }

    fn rebuild_ui_overlay(&mut self, kind: UiOverlayKind) {
        let text_empty = {
            let overlay = self.overlay(kind);
            overlay.text.trim().is_empty()
        };
        if text_empty {
            self.overlay_mut(kind).reset_geometry();
            return;
        }

        let text = {
            let overlay = self.overlay(kind);
            overlay.text.clone()
        };

        if let Some((data, width, height)) = Self::rasterize_ui_text(&text) {
            let texture = Self::create_texture_from_rgba(
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
                width,
                height,
                &data,
                Some("UI Text Texture"),
            );
            {
                let overlay = self.overlay_mut(kind);
                overlay.texture = Some(texture);
                overlay.texture_size = (width, height);
            }
            self.rebuild_ui_vertices_for(kind);
        } else {
            self.overlay_mut(kind).reset_geometry();
        }
    }

    fn rebuild_ui_vertices_for(&mut self, kind: UiOverlayKind) {
        let (tex_w, tex_h) = {
            let overlay = self.overlay(kind);
            overlay.texture_size
        };
        if tex_w == 0 || tex_h == 0 {
            self.overlay_mut(kind).reset_geometry();
            return;
        }

        let width = self.size.width.max(1) as f32;
        let height = self.size.height.max(1) as f32;
        let x0 = UI_TEXT_MARGIN_X.max(0.0);
        let x1 = x0 + tex_w as f32;
        let (y0, y1) = match kind {
            UiOverlayKind::Top => {
                let y0 = UI_TEXT_MARGIN_Y.max(0.0);
                (y0, y0 + tex_h as f32)
            }
            UiOverlayKind::Bottom => {
                let max_y = (height - UI_TEXT_MARGIN_Y).max(UI_TEXT_MARGIN_Y + tex_h as f32);
                let y0 = (max_y - tex_h as f32).max(UI_TEXT_MARGIN_Y);
                (y0, y0 + tex_h as f32)
            }
        };

        let vertices = Self::build_ui_quad_vertices(x0, y0, x1, y1, width, height);
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(match kind {
                    UiOverlayKind::Top => "UI Vertex Buffer",
                    UiOverlayKind::Bottom => "UI Bottom Vertex Buffer",
                }),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let overlay = self.overlay_mut(kind);
        overlay.vertex_count = vertices.len() as u32;
        overlay.vertex_buffer = Some(buffer);
    }

    fn overlay(&self, kind: UiOverlayKind) -> &UiOverlay {
        match kind {
            UiOverlayKind::Top => &self.ui_top,
            UiOverlayKind::Bottom => &self.ui_bottom,
        }
    }

    fn overlay_mut(&mut self, kind: UiOverlayKind) -> &mut UiOverlay {
        match kind {
            UiOverlayKind::Top => &mut self.ui_top,
            UiOverlayKind::Bottom => &mut self.ui_bottom,
        }
    }

    pub fn gpu_memory_for_assets(
        &self,
        model_path: Option<&str>,
        texture_path: Option<&str>,
    ) -> GpuMemoryUsage {
        let model_bytes = model_path
            .and_then(|path| self.lookup_model_bytes(path, texture_path))
            .unwrap_or(0);
        let texture_bytes = texture_path
            .and_then(|path| self.textures.get(path).map(|entry| entry.byte_size))
            .unwrap_or(0);
        GpuMemoryUsage {
            model_bytes,
            texture_bytes,
        }
    }

    fn lookup_model_bytes(&self, model_path: &str, texture_path: Option<&str>) -> Option<usize> {
        let exact_key = ModelKey {
            model: model_path.to_string(),
            texture: texture_path.map(|path| path.to_string()),
        };
        if let Some(entry) = self.models.get(&exact_key) {
            return Some(entry.vertex_bytes + entry.index_bytes);
        }
        self.models
            .iter()
            .find(|(key, _)| key.model == model_path)
            .map(|(_, entry)| entry.vertex_bytes + entry.index_bytes)
    }

    fn build_ui_quad_vertices(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        height: f32,
    ) -> [UiVertex; 6] {
        let top_left = screen_to_ndc(x0, y0, width, height);
        let top_right = screen_to_ndc(x1, y0, width, height);
        let bottom_left = screen_to_ndc(x0, y1, width, height);
        let bottom_right = screen_to_ndc(x1, y1, width, height);

        [
            UiVertex {
                position: [top_left[0], top_left[1], 0.0],
                uv: [0.0, 0.0],
            },
            UiVertex {
                position: [top_right[0], top_right[1], 0.0],
                uv: [1.0, 0.0],
            },
            UiVertex {
                position: [bottom_right[0], bottom_right[1], 0.0],
                uv: [1.0, 1.0],
            },
            UiVertex {
                position: [top_left[0], top_left[1], 0.0],
                uv: [0.0, 0.0],
            },
            UiVertex {
                position: [bottom_right[0], bottom_right[1], 0.0],
                uv: [1.0, 1.0],
            },
            UiVertex {
                position: [bottom_left[0], bottom_left[1], 0.0],
                uv: [0.0, 1.0],
            },
        ]
    }

    fn rasterize_ui_text(text: &str) -> Option<(Vec<u8>, u32, u32)> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return None;
        }
        let max_chars = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        if max_chars == 0 {
            return None;
        }

        let glyph_w = UI_FONT_WIDTH * UI_FONT_SCALE;
        let glyph_h = UI_FONT_HEIGHT * UI_FONT_SCALE;
        let char_spacing = UI_CHAR_SPACING * UI_FONT_SCALE;
        let line_spacing = UI_LINE_SPACING * UI_FONT_SCALE;
        let advance_x = glyph_w + char_spacing;
        let advance_y = glyph_h + line_spacing;

        let width = (UI_TEXT_PADDING_X * 2)
            + if max_chars == 0 {
                0
            } else {
                max_chars * advance_x - char_spacing
            };
        let height = (UI_TEXT_PADDING_Y * 2)
            + if lines.is_empty() {
                0
            } else {
                lines.len() * advance_y - line_spacing
            };

        if width == 0 || height == 0 {
            return None;
        }

        let mut data = vec![0u8; width * height * 4];
        for (line_idx, line) in lines.iter().enumerate() {
            for (char_idx, ch) in line.chars().enumerate() {
                let bits = glyph_bits(ch);
                let origin_x = UI_TEXT_PADDING_X + char_idx * advance_x;
                let origin_y = UI_TEXT_PADDING_Y + line_idx * advance_y;
                for row in 0..UI_FONT_HEIGHT {
                    let row_bits = bits[row];
                    for col in 0..UI_FONT_WIDTH {
                        if (row_bits >> (UI_FONT_WIDTH - 1 - col)) & 1 == 1 {
                            for sy in 0..UI_FONT_SCALE {
                                for sx in 0..UI_FONT_SCALE {
                                    let px = origin_x + col * UI_FONT_SCALE + sx;
                                    let py = origin_y + row * UI_FONT_SCALE + sy;
                                    if px >= width || py >= height {
                                        continue;
                                    }
                                    let idx = (py * width + px) * 4;
                                    data[idx] = 255;
                                    data[idx + 1] = 255;
                                    data[idx + 2] = 255;
                                    data[idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }

        Some((data, width as u32, height as u32))
    }

    fn create_default_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        rgba: [u8; 4],
    ) -> TextureEntry {
        Self::create_texture_from_rgba(device, queue, layout, 1, 1, &rgba, Some("Default Texture"))
    }

    fn update_camera_uniform(&mut self) {
        let view_proj = self.projection * self.view_matrix;
        self.camera_uniform.view_proj = view_proj.to_cols_array_2d();
        self.camera_uniform.eye_position = [
            self.camera_position[0],
            self.camera_position[1],
            self.camera_position[2],
            1.0,
        ];
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera_uniform),
        );
    }

    pub fn update_camera(&mut self, position: &Position, camera: &CameraComponent) {
        let forward = Vec3::new(
            camera.pitch.cos() * camera.yaw.sin(),
            camera.pitch.sin(),
            -camera.pitch.cos() * camera.yaw.cos(),
        )
        .normalize();
        let eye = Vec3::new(position.x, position.y, position.z);
        let up = Vec3::Y;
        self.camera_position = [position.x, position.y, position.z];
        self.view_matrix = Mat4::look_to_rh(eye, forward, up);
        self.update_camera_uniform();
    }

    pub fn update_lighting(&mut self, ecs: &ECS) {
        let mut directional = None;
        let mut point_index = 0;
        for (position, light) in ecs.light_components() {
            match light.kind {
                LightKind::Directional(direction) => {
                    if directional.is_none() {
                        directional = Some((direction, light.color, light.intensity));
                    }
                }
                LightKind::Point { radius } => {
                    if point_index < MAX_POINT_LIGHTS {
                        self.light_uniform.point_positions[point_index] = [
                            position.x,
                            position.y,
                            position.z,
                            radius.max(0.001),
                        ];
                        self.light_uniform.point_colors[point_index] = [
                            light.color[0],
                            light.color[1],
                            light.color[2],
                            light.intensity,
                        ];
                        point_index += 1;
                    }
                }
            }
        }
        if let Some((direction, color, intensity)) = directional {
            self.light_uniform.directional_direction =
                [direction[0], direction[1], direction[2], 0.0];
            self.light_uniform.directional_color =
                [color[0], color[1], color[2], intensity];
        } else {
            self.light_uniform.directional_direction = [0.0, -1.0, 0.0, 0.0];
            self.light_uniform.directional_color = [1.0, 1.0, 1.0, 0.0];
        }
        for i in point_index..MAX_POINT_LIGHTS {
            self.light_uniform.point_positions[i] = [0.0, 0.0, 0.0, 0.0];
            self.light_uniform.point_colors[i] = [0.0, 0.0, 0.0, 0.0];
        }
        self.light_uniform.point_count = [point_index as u32, 0, 0, 0];
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::bytes_of(&self.light_uniform),
        );
    }

    pub fn set_background_colors(&mut self, top: [f32; 3], bottom: [f32; 3]) {
        let uniform = BackgroundUniform::new(top, bottom);
        self.queue
            .write_buffer(&self.background_buffer, 0, bytemuck::bytes_of(&uniform));
        self.clear_color = bottom;
    }

    pub fn set_fog(&mut self, color: [f32; 3], density: f32) {
        self.fog_uniform.color = [color[0], color[1], color[2], 1.0];
        self.fog_uniform.params = [density, 0.0, 0.0, 0.0];
        self.queue
            .write_buffer(&self.fog_buffer, 0, bytemuck::bytes_of(&self.fog_uniform));
    }

    pub fn set_ui_text<S: Into<String>>(&mut self, text: S) {
        self.set_ui_overlay_text(UiOverlayKind::Top, text.into());
    }

    pub fn set_bottom_ui_text<S: Into<String>>(&mut self, text: S) {
        self.set_ui_overlay_text(UiOverlayKind::Bottom, text.into());
    }

    fn set_ui_overlay_text(&mut self, kind: UiOverlayKind, text: String) {
        self.overlay_mut(kind).text = text;
        self.rebuild_ui_overlay(kind);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut bg_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color[0] as f64,
                            g: self.clear_color[1] as f64,
                            b: self.clear_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            bg_pass.set_pipeline(&self.background_pipeline);
            bg_pass.set_bind_group(0, &self.background_bind_group, &[]);
            bg_pass.set_vertex_buffer(0, self.background_vertex_buffer.slice(..));
            bg_pass.draw(0..FULLSCREEN_VERTICES.len() as u32, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for entry in self.models.values() {
                if entry.instance_count == 0 {
                    continue;
                }
                render_pass.set_bind_group(1, entry.texture_bind_group.as_ref(), &[]);
                let instance_buffer = match &entry.instance_buffer {
                    Some(buffer) => buffer,
                    None => continue,
                };
                render_pass.set_vertex_buffer(0, entry.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(entry.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..entry.index_count, 0, 0..entry.instance_count);
            }
        }

        if self.debug_line_vertex_count > 0 {
            if let Some(buffer) = &self.debug_line_buffer {
                let mut line_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Debug Line Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                line_pass.set_pipeline(&self.debug_line_pipeline);
                line_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                line_pass.set_vertex_buffer(0, buffer.slice(..));
                line_pass.draw(0..self.debug_line_vertex_count, 0..1);
            }
        }

        for (overlay, label) in [
            (&self.ui_top, "UI Top Pass"),
            (&self.ui_bottom, "UI Bottom Pass"),
        ] {
            if overlay.vertex_count == 0 {
                continue;
            }
            if let (Some(buffer), Some(texture)) =
                (&overlay.vertex_buffer, overlay.texture.as_ref())
            {
                let mut ui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(label),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                ui_pass.set_pipeline(&self.ui_pipeline);
                ui_pass.set_bind_group(0, texture.bind_group.as_ref(), &[]);
                ui_pass.set_vertex_buffer(0, buffer.slice(..));
                ui_pass.draw(0..overlay.vertex_count, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn build_projection_matrix(size: PhysicalSize<u32>) -> Mat4 {
    let aspect = size.width.max(1) as f32 / size.height.max(1) as f32;
    Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0)
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> DepthTexture {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    DepthTexture { view }
}

fn screen_to_ndc(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    let ndc_x = (x / width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / height) * 2.0;
    [ndc_x, ndc_y]
}
