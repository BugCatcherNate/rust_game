use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use crate::components::{
    CameraComponent, PhysicsBodyType, PhysicsComponent, Position, ScriptComponent,
};
use crate::ecs::{ComponentKind, ECS};
use crate::rendering::{DebugGizmo, DebugLine, Renderer};
use crate::scene::{self, SceneLibrary, SceneLookupError};
use crate::scripts::{ScriptCommand, ScriptRegistry};
use crate::systems::{
    CameraSystem, HierarchySystem, InputSystem, MovementSystem, ParticleSystem, PhysicsSystem,
    RenderPrepSystem, ScriptingSystem,
};
use log::{info, warn};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, DeviceId, Ime, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

const CONSOLE_HISTORY_LIMIT: usize = 10;

/// Configuration for running the built-in engine loop.
pub struct GameConfig {
    pub initial_scene: String,
    pub scenes: SceneLibrary,
    pub window_title: String,
    pub script_registry: Arc<ScriptRegistry>,
    pub script_bindings: Vec<ScriptBinding>,
    pub custom_systems: Vec<Box<dyn CustomSystem>>,
}

#[derive(Debug, Clone)]
pub struct ShotEvent {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub hit: Option<ShotEventHit>,
}

#[derive(Debug, Clone)]
pub struct ShotEventHit {
    pub entity_id: u32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct ConsoleCommandEvent {
    pub text: String,
}

impl GameConfig {
    pub fn new(
        initial_scene: impl Into<String>,
        scenes: SceneLibrary,
        script_registry: Arc<ScriptRegistry>,
    ) -> Self {
        let initial_scene = initial_scene.into();
        if !scenes.contains(initial_scene.as_str()) {
            panic!(
                "Initial scene '{}' not found in provided scene library",
                initial_scene
            );
        }
        Self {
            initial_scene,
            scenes,
            window_title: "Rust ECS Demo".to_string(),
            script_registry,
            script_bindings: Vec::new(),
            custom_systems: Vec::new(),
        }
    }

    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.window_title = title.into();
        self
    }

    pub fn with_script_binding(mut self, binding: ScriptBinding) -> Self {
        self.script_bindings.push(binding);
        self
    }

    pub fn with_script_bindings<I>(mut self, bindings: I) -> Self
    where
        I: IntoIterator<Item = ScriptBinding>,
    {
        self.script_bindings.extend(bindings);
        self
    }

    pub fn with_custom_system<S>(mut self, system: S) -> Self
    where
        S: CustomSystem + 'static,
    {
        self.custom_systems.push(Box::new(system));
        self
    }

    pub fn with_custom_systems<I, S>(mut self, systems: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: CustomSystem + 'static,
    {
        self.custom_systems.extend(
            systems
                .into_iter()
                .map(|s| Box::new(s) as Box<dyn CustomSystem>),
        );
        self
    }
}

#[derive(Debug, Clone)]
pub struct ScriptBinding {
    pub entity_name: String,
    pub script: String,
    pub params: HashMap<String, String>,
}

pub trait CustomSystem: Send {
    fn scene_loaded(&mut self, _ecs: &mut ECS, _scene: &str) {}
    fn update(&mut self, ecs: &mut ECS, current_scene: &str, commands: &mut Vec<ScriptCommand>);
    fn hud_text(&mut self, _ecs: &ECS, _current_scene: &str) -> Option<String> {
        None
    }
    fn before_fire(&mut self, _ecs: &mut ECS) -> bool {
        true
    }
}

impl ScriptBinding {
    pub fn new(entity_name: impl Into<String>, script: impl Into<String>) -> Self {
        Self {
            entity_name: entity_name.into(),
            script: script.into(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// Run the engine-provided application loop using the supplied configuration.
pub fn run_game(config: GameConfig) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()
        .map_err(|e: EventLoopError| -> Box<dyn std::error::Error> { Box::new(e) })?;
    let mut app = GameApp::new(config);
    event_loop
        .run_app(&mut app)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    Ok(())
}

struct GameApp {
    ecs: ECS,
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
    pressed_keys: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    player_id: u32,
    camera_id: u32,
    printed_state: bool,
    shutdown_called: bool,
    pending_mouse_delta: (f64, f64),
    scene_settings: scene::SceneSettings,
    scripting_system: ScriptingSystem,
    physics_system: PhysicsSystem,
    audio_stream: Option<OutputStream>,
    audio_handle: Option<OutputStreamHandle>,
    audio_sink: Option<Sink>,
    current_scene_id: String,
    scene_library: SceneLibrary,
    window_title: String,
    script_bindings: Vec<ScriptBinding>,
    custom_systems: Vec<Box<dyn CustomSystem>>,
    custom_command_buffer: Vec<ScriptCommand>,
    debug_lines: Vec<DebugLine>,
    debug_gizmos: Vec<DebugGizmo>,
    debug_text_labels: Vec<String>,
    debug_lines_enabled: bool,
    console_open: bool,
    console_input: String,
    console_history: VecDeque<String>,
    creative_mode: bool,
    saved_player_physics: Option<PhysicsComponent>,
    saved_camera_physics: Option<PhysicsComponent>,
}

impl GameApp {
    fn new(config: GameConfig) -> Self {
        let GameConfig {
            initial_scene,
            scenes,
            window_title,
            script_registry,
            script_bindings,
            custom_systems,
        } = config;
        let mut app = Self {
            ecs: ECS::new(),
            renderer: None,
            window: None,
            window_id: None,
            pressed_keys: HashSet::new(),
            just_pressed: HashSet::new(),
            player_id: 0,
            camera_id: 0,
            printed_state: false,
            shutdown_called: false,
            pending_mouse_delta: (0.0, 0.0),
            scene_settings: scene::SceneSettings::default(),
            scripting_system: ScriptingSystem::new(script_registry),
            physics_system: PhysicsSystem::new(),
            audio_stream: None,
            audio_handle: None,
            audio_sink: None,
            current_scene_id: initial_scene.clone(),
            scene_library: scenes,
            window_title,
            script_bindings,
            custom_systems,
            custom_command_buffer: Vec::new(),
            debug_lines: Vec::new(),
            debug_gizmos: Vec::new(),
            debug_text_labels: Vec::new(),
            debug_lines_enabled: true,
            console_open: false,
            console_input: String::new(),
            console_history: VecDeque::new(),
            creative_mode: false,
            saved_player_physics: None,
            saved_camera_physics: None,
        };
        app.reload_scene(&initial_scene)
            .unwrap_or_else(|err| panic!("Failed to load scene '{}': {}", initial_scene, err));
        app
    }

    fn require_tagged_entity(ecs: &ECS, tag: &str) -> u32 {
        let set = ecs
            .tag_manager
            .get_entities_with_tag(tag)
            .unwrap_or_else(|| panic!("Scene missing entity tagged '{}'", tag));
        let mut iter = set.iter();
        let id = iter
            .next()
            .copied()
            .unwrap_or_else(|| panic!("Scene missing entity tagged '{}'", tag));
        if iter.next().is_some() {
            log::warn!("Multiple entities tagged '{}'; using {:?}", tag, id);
        }
        id
    }

    fn build_hud_text(position: &Position, camera: &CameraComponent) -> String {
        format!(
            "Rust ECS Demo - WASD to move\nPos: ({:.2}, {:.2}, {:.2})  Yaw: {:.2}  Pitch: {:.2}",
            position.x, position.y, position.z, camera.yaw, camera.pitch
        )
    }

    fn toggle_console(&mut self) {
        self.console_open = !self.console_open;
        if self.console_open {
            self.pressed_keys.clear();
            self.just_pressed.clear();
        }
    }

    fn hide_console(&mut self) {
        self.console_open = false;
    }

    fn push_console_history(&mut self, entry: String) {
        self.console_history.push_back(entry);
        while self.console_history.len() > CONSOLE_HISTORY_LIMIT {
            self.console_history.pop_front();
        }
    }

    fn submit_console_command(&mut self) {
        let trimmed = self.console_input.trim();
        if !trimmed.is_empty() {
            let command = trimmed.to_string();
            self.push_console_history(command.clone());
            self.handle_console_command(&command);
            self.ecs.emit_event(ConsoleCommandEvent { text: command });
        }
        self.console_input.clear();
    }

    fn handle_console_command(&mut self, command: &str) {
        let normalized = command.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return;
        }
        match normalized.as_str() {
            "mem" | "memory" | "memory usage" => {
                self.display_memory_usage();
                return;
            }
            "creative" => {
                self.toggle_creative_mode();
                return;
            }
            _ => {}
        }

        let mut parts = normalized.split_whitespace();
        if let Some(keyword) = parts.next() {
            match keyword {
                "debuglines" => self.handle_debug_lines_command(parts.next()),
                _ => {}
            }
        }
    }

    fn handle_debug_lines_command(&mut self, argument: Option<&str>) {
        match argument {
            Some("on") => self.set_debug_lines_enabled(true),
            Some("off") => self.set_debug_lines_enabled(false),
            Some("toggle") | None => {
                let enabled = !self.debug_lines_enabled;
                self.set_debug_lines_enabled(enabled);
            }
            Some(other) => {
                self.push_console_history(format!(
                    "debuglines usage: debuglines [on|off|toggle] (got '{}')",
                    other
                ));
            }
        }
    }

    fn set_debug_lines_enabled(&mut self, enabled: bool) {
        self.debug_lines_enabled = enabled;
        let status = if enabled { "enabled" } else { "disabled" };
        self.push_console_history(format!("Debug lines {}", status));
    }

    fn display_memory_usage(&mut self) {
        let cpu_bytes = self.ecs.total_memory_usage();
        let gpu_bytes = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.total_gpu_memory())
            .unwrap_or(0);
        let message = format!(
            "Memory: CPU {} | GPU {}",
            Self::format_bytes(cpu_bytes),
            Self::format_bytes(gpu_bytes)
        );
        self.push_console_history(message);
    }

    fn format_bytes(bytes: usize) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut value = bytes as f64;
        let mut unit_index = 0;
        while value >= 1024.0 && unit_index < UNITS.len() - 1 {
            value /= 1024.0;
            unit_index += 1;
        }
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", value, UNITS[unit_index])
        }
    }

    fn toggle_creative_mode(&mut self) {
        if self.creative_mode {
            self.disable_creative_mode();
        } else {
            self.enable_creative_mode();
        }
    }

    fn enable_creative_mode(&mut self) {
        if self.creative_mode {
            return;
        }
        self.saved_player_physics = self.ecs.physics_component(self.player_id).cloned();
        if self.saved_player_physics.is_some() {
            self.ecs.remove_physics_component(self.player_id);
        }
        if self.camera_id != self.player_id {
            self.saved_camera_physics = self.ecs.physics_component(self.camera_id).cloned();
            if self.saved_camera_physics.is_some() {
                self.ecs.remove_physics_component(self.camera_id);
            }
        } else {
            self.saved_camera_physics = None;
        }
        self.physics_system.rebuild_from_ecs(&mut self.ecs);
        self.creative_mode = true;
        self.push_console_history("Creative mode enabled".to_string());
    }

    fn disable_creative_mode(&mut self) {
        if !self.creative_mode {
            return;
        }
        if let Some(component) = self.saved_player_physics.take() {
            self.ecs.add_physics_component(self.player_id, component);
        }
        if let Some(component) = self.saved_camera_physics.take() {
            if self.camera_id != self.player_id {
                self.ecs.add_physics_component(self.camera_id, component);
            } else {
                self.ecs.add_physics_component(self.player_id, component);
            }
        }
        self.physics_system.rebuild_from_ecs(&mut self.ecs);
        self.creative_mode = false;
        self.push_console_history("Creative mode disabled".to_string());
    }

    fn console_overlay_text(&self) -> String {
        let mut lines = Vec::new();
        lines.push("Debug Console (` to close, Enter to submit)".to_string());
        for entry in self.console_history.iter() {
            lines.push(format!("> {}", entry));
        }
        lines.push(format!("> {}_", self.console_input));
        lines.join("\n")
    }

    fn ensure_shutdown(&mut self) {
        if !self.shutdown_called {
            self.stop_background_audio();
            crate::modules::core::shutdown();
            self.shutdown_called = true;
        }
    }

    fn start_background_audio(&mut self) {
        if self.audio_sink.is_some() {
            return;
        }
        let Some(path) = self.scene_settings.background_sound.as_ref() else {
            return;
        };
        let Ok((stream, handle)) = OutputStream::try_default() else {
            warn!("Failed to initialize audio output");
            return;
        };
        let Ok(file) = File::open(path) else {
            warn!("Failed to open background sound '{}': file not found", path);
            return;
        };
        let Ok(decoder) = Decoder::new(BufReader::new(file)) else {
            warn!(
                "Failed to decode background sound '{}': unsupported format",
                path
            );
            return;
        };
        let Ok(sink) = Sink::try_new(&handle) else {
            warn!("Failed to create audio sink");
            return;
        };
        sink.append(decoder.repeat_infinite());
        sink.play();
        self.audio_stream = Some(stream);
        self.audio_handle = Some(handle);
        self.audio_sink = Some(sink);
    }

    fn stop_background_audio(&mut self) {
        if let Some(sink) = self.audio_sink.take() {
            sink.stop();
        }
        self.audio_handle = None;
        self.audio_stream = None;
    }

    fn lock_cursor(&self) {
        if let Some(window) = &self.window {
            let modes = [CursorGrabMode::Locked, CursorGrabMode::Confined];
            let mut locked = false;
            for mode in modes {
                match window.set_cursor_grab(mode) {
                    Ok(()) => {
                        locked = true;
                        break;
                    }
                    Err(err) => {
                        warn!("Failed to set cursor grab mode {:?}: {}", mode, err);
                    }
                }
            }
            if locked {
                window.set_cursor_visible(false);
            } else {
                warn!("Falling back to visible cursor; grab not supported");
                window.set_cursor_visible(true);
            }
        }
    }

    fn unlock_cursor(&self) {
        if let Some(window) = &self.window {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }

    fn log_entity_state_once(&mut self) {
        if self.printed_state {
            return;
        }
        if let Some((position, _, name)) = self.ecs.find_entity_components(self.player_id) {
            info!(
                "After movement, entity {:?} is at position {:?}",
                name, position
            );
        }
        if let Some((position, _, name)) = self.ecs.find_entity_components(self.camera_id) {
            info!(
                "After movement, entity {:?} is at position {:?}",
                name, position
            );
        }
        if let Some(entities) = self.ecs.tag_manager.get_entities_with_tag("player") {
            info!("Entities with player tag: {:?}", entities);
        }
        if let Some(entities) = self.ecs.tag_manager.get_entities_with_tag("camera") {
            info!("Entities with camera tag: {:?}", entities);
        }
        info!("-- Entity Memory Usage (approximate) --");
        let renderer = self.renderer.as_ref();
        for report in self.ecs.entity_memory_usage() {
            let component_breakdown = report
                .components
                .iter()
                .map(|segment| format!("{}: {}B", segment.label, segment.estimated_bytes))
                .collect::<Vec<_>>()
                .join(", ");
            let mut line = format!(
                "{} (id {}): ~{}B [{}]",
                report.name, report.id, report.estimated_bytes, component_breakdown
            );
            if let Some(renderer) = renderer {
                let model_path = self
                    .ecs
                    .model_component(report.id)
                    .map(|component| component.asset_path.as_str());
                let texture_path = self
                    .ecs
                    .texture_component(report.id)
                    .map(|component| component.asset_path.as_str());
                let gpu = renderer.gpu_memory_for_assets(model_path, texture_path);
                if gpu.model_bytes != 0 || gpu.texture_bytes != 0 {
                    line.push_str(&format!(
                        " | GPU ~{}B (model {}B, texture {}B)",
                        gpu.total_bytes(),
                        gpu.model_bytes,
                        gpu.texture_bytes
                    ));
                }
            }
            info!("{}", line);
        }
        let total = self.ecs.total_memory_usage();
        if let Some(renderer) = renderer {
            info!(
                "Total GPU memory (models + textures): {}B",
                renderer.total_gpu_memory()
            );
        }
        info!("Total approximate memory: {}B", total);
        self.printed_state = true;
    }

    fn handle_render_error(&mut self, event_loop: &ActiveEventLoop, error: wgpu::SurfaceError) {
        match error {
            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(renderer.size());
                }
            }
            wgpu::SurfaceError::OutOfMemory => {
                self.ensure_shutdown();
                event_loop.exit();
            }
            wgpu::SurfaceError::Timeout => {
                log::warn!("Surface timeout, skipping frame");
            }
        }
    }

    fn reload_scene(&mut self, scene_id: &str) -> Result<(), SceneLookupError> {
        let scene_definition = self
            .scene_library
            .get(scene_id)
            .ok_or_else(|| SceneLookupError::NotFound(scene_id.to_string()))?;
        let mut ecs = ECS::new();
        let scene_settings = scene::apply_scene_definition(scene_definition, &mut ecs);
        let player_id = Self::require_tagged_entity(&ecs, "player");
        let camera_id = Self::require_tagged_entity(&ecs, "camera");

        self.ecs = ecs;
        self.scene_settings = scene_settings;
        self.player_id = player_id;
        self.camera_id = camera_id;
        self.current_scene_id = scene_id.to_string();
        self.printed_state = false;
        self.pending_mouse_delta = (0.0, 0.0);
        self.creative_mode = false;
        self.saved_player_physics = None;
        self.saved_camera_physics = None;

        self.scripting_system.reset();

        if self.audio_sink.is_some() {
            self.restart_background_audio();
        }

        self.apply_script_bindings();
        self.physics_system.rebuild_from_ecs(&mut self.ecs);
        HierarchySystem::update(&mut self.ecs);
        for system in self.custom_systems.iter_mut() {
            system.scene_loaded(&mut self.ecs, scene_id);
        }
        self.update_renderer_scene();

        Ok(())
    }

    fn restart_background_audio(&mut self) {
        self.stop_background_audio();
        self.start_background_audio();
    }

    fn handle_fire(&mut self) {
        for system in self.custom_systems.iter_mut() {
            if !system.before_fire(&mut self.ecs) {
                return;
            }
        }
        let Some((position, _, _)) = self.ecs.find_entity_components(self.camera_id) else {
            return;
        };
        let Some(camera) = self.ecs.camera_component(self.camera_id) else {
            return;
        };
        let origin = position.as_array();
        let direction = Self::camera_forward(camera);
        let mut ray_origin = origin;
        for i in 0..3 {
            ray_origin[i] += direction[i] * 2.0;
        }
        let max_distance = 100.0;
        let mut line_end = [
            origin[0] + direction[0] * max_distance,
            origin[1] + direction[1] * max_distance,
            origin[2] + direction[2] * max_distance,
        ];
        let mut line_color = [1.0, 1.0, 0.2];
        let mut hit_result = None;
        if let Some(hit) = self
            .physics_system
            .cast_ray(ray_origin, direction, max_distance)
        {
            line_end = hit.point;
            line_color = [1.0, 0.2, 0.2];
            info!(
                "Shot hit entity {} at ({:.2}, {:.2}, {:.2})",
                hit.entity_id, hit.point[0], hit.point[1], hit.point[2]
            );
            self.debug_gizmos.push(DebugGizmo::WireSphere {
                center: hit.point,
                radius: 0.25,
                color: [1.0, 0.3, 0.3],
            });
            self.debug_gizmos.push(DebugGizmo::Label {
                position: hit.point,
                text: format!("Hit {}", hit.entity_id),
            });
            hit_result = Some(ShotEventHit {
                entity_id: hit.entity_id,
                point: hit.point,
                normal: hit.normal,
            });
        } else {
            info!("Shot missed");
        }
        self.debug_lines.push(DebugLine {
            start: origin,
            end: line_end,
            color: line_color,
        });
        self.ecs.emit_event(ShotEvent {
            origin,
            direction,
            hit: hit_result,
        });
    }

    fn queue_physics_gizmos(&mut self) {
        for archetype in &self.ecs.archetypes {
            let Some(physics_components) = &archetype.physics else {
                continue;
            };
            for (index, physics) in physics_components.iter().enumerate() {
                let position = archetype.positions[index];
                let color = Self::debug_color_for_body(physics.body_type);
                self.debug_gizmos.push(DebugGizmo::WireBox {
                    center: position.as_array(),
                    half_extents: physics.half_extents,
                    color,
                });
            }
        }
    }

    fn flush_debug_gizmos(&mut self) {
        if !self.debug_lines_enabled {
            self.debug_gizmos.clear();
            self.debug_text_labels.clear();
            return;
        }
        if self.debug_gizmos.is_empty() {
            self.debug_text_labels.clear();
            return;
        }
        let mut derived_lines = Vec::new();
        self.debug_text_labels.clear();
        for gizmo in self.debug_gizmos.drain(..) {
            match gizmo {
                DebugGizmo::WireBox {
                    center,
                    half_extents,
                    color,
                } => Self::add_box_lines(&mut derived_lines, center, half_extents, color),
                DebugGizmo::WireSphere {
                    center,
                    radius,
                    color,
                } => Self::add_sphere_lines(&mut derived_lines, center, radius, color),
                DebugGizmo::Label { position, text } => {
                    self.debug_text_labels.push(format!(
                        "{} @ ({:.2}, {:.2}, {:.2})",
                        text, position[0], position[1], position[2]
                    ));
                }
            }
        }
        self.debug_lines.extend(derived_lines);
    }

    fn add_box_lines(
        lines: &mut Vec<DebugLine>,
        center: [f32; 3],
        half_extents: [f32; 3],
        color: [f32; 3],
    ) {
        let [hx, hy, hz] = half_extents;
        let corners = [
            [center[0] - hx, center[1] - hy, center[2] - hz],
            [center[0] + hx, center[1] - hy, center[2] - hz],
            [center[0] + hx, center[1] - hy, center[2] + hz],
            [center[0] - hx, center[1] - hy, center[2] + hz],
            [center[0] - hx, center[1] + hy, center[2] - hz],
            [center[0] + hx, center[1] + hy, center[2] - hz],
            [center[0] + hx, center[1] + hy, center[2] + hz],
            [center[0] - hx, center[1] + hy, center[2] + hz],
        ];
        const EDGES: [(usize, usize); 12] = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        for &(start, end) in &EDGES {
            lines.push(DebugLine {
                start: corners[start],
                end: corners[end],
                color,
            });
        }
    }

    fn debug_color_for_body(body_type: PhysicsBodyType) -> [f32; 3] {
        match body_type {
            PhysicsBodyType::Dynamic => [0.2, 0.8, 1.0],
            PhysicsBodyType::Static => [0.3, 1.0, 0.3],
            PhysicsBodyType::Kinematic => [1.0, 0.8, 0.3],
        }
    }

    fn add_sphere_lines(
        lines: &mut Vec<DebugLine>,
        center: [f32; 3],
        radius: f32,
        color: [f32; 3],
    ) {
        const SEGMENTS: usize = 32;
        for plane in [CirclePlane::XY, CirclePlane::XZ, CirclePlane::YZ] {
            Self::add_circle(lines, center, radius, color, plane, SEGMENTS);
        }
    }

    fn add_circle(
        lines: &mut Vec<DebugLine>,
        center: [f32; 3],
        radius: f32,
        color: [f32; 3],
        plane: CirclePlane,
        segments: usize,
    ) {
        let step = std::f32::consts::TAU / segments as f32;
        let mut prev = None;
        for i in 0..=segments {
            let angle = i as f32 * step;
            let mut point = center;
            let (sin, cos) = angle.sin_cos();
            match plane {
                CirclePlane::XY => {
                    point[0] += radius * cos;
                    point[1] += radius * sin;
                }
                CirclePlane::XZ => {
                    point[0] += radius * cos;
                    point[2] += radius * sin;
                }
                CirclePlane::YZ => {
                    point[1] += radius * cos;
                    point[2] += radius * sin;
                }
            }
            if let Some(previous) = prev {
                lines.push(DebugLine {
                    start: previous,
                    end: point,
                    color,
                });
            }
            prev = Some(point);
        }
    }

    fn camera_forward(camera: &CameraComponent) -> [f32; 3] {
        let cos_pitch = camera.pitch.cos();
        [
            cos_pitch * camera.yaw.sin(),
            camera.pitch.sin(),
            -cos_pitch * camera.yaw.cos(),
        ]
    }

    fn update_renderer_scene(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_background_colors(
                self.scene_settings.background_top,
                self.scene_settings.background_bottom,
            );
            renderer.set_fog(
                self.scene_settings.fog_color,
                self.scene_settings.fog_density,
            );
            if let Some((position, _, _)) = self.ecs.find_entity_components(self.camera_id) {
                if let Some(camera) = self.ecs.camera_component(self.camera_id) {
                    renderer.update_camera(position, camera);
                    renderer.set_ui_text(Self::build_hud_text(position, camera));
                }
            }
            renderer.set_bottom_ui_text(String::new());
            RenderPrepSystem::update(renderer, &self.ecs, Some(self.camera_id));
        }
    }

    fn apply_script_bindings(&mut self) {
        if self.script_bindings.is_empty() {
            return;
        }
        for binding in &self.script_bindings {
            let Some(entity_id) = self.ecs.find_entity_id_by_name(&binding.entity_name) else {
                warn!(
                    "Script binding '{}' -> '{}' skipped: entity not found",
                    binding.entity_name, binding.script
                );
                continue;
            };
            let base_height = self
                .ecs
                .find_entity_components(entity_id)
                .map(|(position, _, _)| position.y)
                .unwrap_or(0.0);
            let component = ScriptComponent::with_params(
                binding.script.clone(),
                base_height,
                binding.params.clone(),
            );
            self.ecs.add_script_component(entity_id, component);
        }
    }

    fn process_commands<I>(&mut self, commands: I) -> bool
    where
        I: IntoIterator<Item = ScriptCommand>,
    {
        let mut scene_changed = false;
        for command in commands {
            match command {
                ScriptCommand::LoadScene(path) => {
                    if path == self.current_scene_id {
                        continue;
                    }
                    if let Err(err) = self.reload_scene(&path) {
                        warn!("Failed to load scene '{}': {}", path, err);
                    } else {
                        info!("Loaded scene '{}'", path);
                        scene_changed = true;
                    }
                }
                ScriptCommand::RemoveComponent {
                    entity_id,
                    component,
                } => {
                    self.apply_component_removal(entity_id, component);
                }
                ScriptCommand::EmitEvent(event) => {
                    self.ecs.emit_boxed_event(event);
                }
                ScriptCommand::DebugLine(line) => {
                    self.debug_lines.push(line);
                }
                ScriptCommand::DebugGizmo(gizmo) => {
                    self.debug_gizmos.push(gizmo);
                }
            }
        }
        scene_changed
    }

    fn apply_component_removal(&mut self, entity_id: u32, component: ComponentKind) {
        match component {
            ComponentKind::Render => self.ecs.remove_render_component(entity_id),
            ComponentKind::Input => self.ecs.remove_input_component(entity_id),
            ComponentKind::Model => self.ecs.remove_model_component(entity_id),
            ComponentKind::Camera => self.ecs.remove_camera_component(entity_id),
            ComponentKind::Light => self.ecs.remove_light_component(entity_id),
            ComponentKind::Texture => self.ecs.remove_texture_component(entity_id),
            ComponentKind::Terrain => self.ecs.remove_terrain_component(entity_id),
            ComponentKind::Script => self.ecs.remove_script_component(entity_id),
            ComponentKind::Physics => self.ecs.remove_physics_component(entity_id),
            ComponentKind::Hierarchy => self.ecs.remove_hierarchy_component(entity_id),
            ComponentKind::Attributes => self.ecs.remove_attributes_component(entity_id),
            ComponentKind::ParticleEmitter => self.ecs.remove_particle_emitter_component(entity_id),
            ComponentKind::Particle => self.ecs.remove_particle_component(entity_id),
        }
    }
}

#[derive(Copy, Clone)]
enum CirclePlane {
    XY,
    XZ,
    YZ,
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = create_game_window(event_loop, &self.window_title);
        window.set_ime_allowed(true);

        self.window_id = Some(window.id());
        self.renderer = Some(pollster::block_on(Renderer::new(window.clone())));
        self.window = Some(window);

        self.update_renderer_scene();
        self.start_background_audio();
        self.lock_cursor();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.ensure_shutdown();
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(new_size);
                }
            }
            WindowEvent::Ime(ime_event) => {
                if self.console_open {
                    if let Ime::Commit(text) = ime_event {
                        self.console_input.push_str(&text);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::Backquote
                        && event.state == winit::event::ElementState::Pressed
                    {
                        self.toggle_console();
                        return;
                    }
                    if code == KeyCode::Escape && event.state == winit::event::ElementState::Pressed
                    {
                        if self.console_open {
                            self.hide_console();
                            return;
                        }
                        self.ensure_shutdown();
                        event_loop.exit();
                        return;
                    }
                    if self.console_open {
                        if event.state == winit::event::ElementState::Pressed {
                            match code {
                                KeyCode::Enter => {
                                    self.submit_console_command();
                                }
                                KeyCode::Backspace => {
                                    self.console_input.pop();
                                }
                                _ => {
                                    if let Some(text) = event.text.as_ref() {
                                        for ch in text.chars() {
                                            if !ch.is_control() {
                                                self.console_input.push(ch);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return;
                    }
                    match event.state {
                        winit::event::ElementState::Pressed => {
                            self.pressed_keys.insert(code);
                            self.just_pressed.insert(code);
                        }
                        winit::event::ElementState::Released => {
                            self.pressed_keys.remove(&code);
                            self.just_pressed.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == winit::event::ElementState::Pressed {
                    self.handle_fire();
                }
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    self.lock_cursor();
                } else {
                    self.unlock_cursor();
                }
            }
            WindowEvent::RedrawRequested => {
                let delta = self.pending_mouse_delta;
                self.pending_mouse_delta = (0.0, 0.0);
                CameraSystem::apply_mouse_delta(&mut self.ecs, self.camera_id, delta);
                InputSystem::update_entity_from_keys(
                    &mut self.ecs,
                    self.player_id,
                    &self.pressed_keys,
                    &self.just_pressed,
                    self.creative_mode,
                );
                if self.camera_id != self.player_id {
                    InputSystem::update_entity_from_keys(
                        &mut self.ecs,
                        self.camera_id,
                        &self.pressed_keys,
                        &self.just_pressed,
                        self.creative_mode,
                    );
                }
                self.just_pressed.clear();
                MovementSystem::update(&mut self.ecs);
                self.physics_system.update(&mut self.ecs);
                HierarchySystem::update(&mut self.ecs);
                for system in self.custom_systems.iter_mut() {
                    system.update(
                        &mut self.ecs,
                        &self.current_scene_id,
                        &mut self.custom_command_buffer,
                    );
                }
                ParticleSystem::update(&mut self.ecs, Some(self.camera_id));
                self.scripting_system.update(&mut self.ecs);
                let mut commands = self.scripting_system.take_commands();
                commands.extend(self.custom_command_buffer.drain(..));
                let _scene_changed = self.process_commands(commands);
                let console_overlay = if self.console_open {
                    self.console_overlay_text()
                } else {
                    String::new()
                };
                if self.debug_lines_enabled {
                    self.queue_physics_gizmos();
                } else {
                    self.debug_gizmos.clear();
                }
                self.flush_debug_gizmos();
                if let Some(renderer) = self.renderer.as_mut() {
                    RenderPrepSystem::update(renderer, &self.ecs, Some(self.camera_id));
                    if self.debug_lines_enabled {
                        renderer.set_debug_lines(&self.debug_lines);
                    } else {
                        renderer.set_debug_lines(&[]);
                    }
                    if let Some((position, _, _)) = self.ecs.find_entity_components(self.camera_id)
                    {
                        if let Some(camera) = self.ecs.camera_component(self.camera_id) {
                            let default_text = Self::build_hud_text(position, camera);
                            let mut custom_sections = Vec::new();
                            for system in self.custom_systems.iter_mut() {
                                if let Some(text) =
                                    system.hud_text(&self.ecs, &self.current_scene_id)
                                {
                                    custom_sections.push(text);
                                }
                            }
                            if custom_sections.is_empty() {
                                renderer.set_ui_text(default_text);
                            } else {
                                custom_sections.push(default_text);
                                renderer.set_ui_text(custom_sections.join("\n\n"));
                            }
                        }
                    }
                    let mut bottom_sections = Vec::new();
                    if !self.debug_text_labels.is_empty() {
                        bottom_sections.push(self.debug_text_labels.join("\n"));
                    }
                    if !console_overlay.is_empty() {
                        bottom_sections.push(console_overlay);
                    }
                    renderer.set_bottom_ui_text(bottom_sections.join("\n\n"));
                    if let Err(err) = renderer.render() {
                        self.handle_render_error(event_loop, err);
                    }
                }
                self.debug_lines.clear();
                self.debug_text_labels.clear();
                self.log_entity_state_once();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if self.window_id.is_some() {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.pending_mouse_delta.0 += delta.0;
                self.pending_mouse_delta.1 += delta.1;
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.ensure_shutdown();
    }
}

impl Drop for GameApp {
    fn drop(&mut self) {
        self.ensure_shutdown();
    }
}

fn create_game_window(event_loop: &ActiveEventLoop, title: &str) -> Arc<Window> {
    let window = Arc::new(
        event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title(title)
                    .with_resizable(true),
            )
            .expect("Failed to create window"),
    );
    window
}
