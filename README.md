# Rust Game / Engine Demo

An experiment-sized first-person game built on a lightweight Rust engine. The workspace contains a reusable `rust_game` engine crate (rendering, ECS, physics, scripting, audio) and a `rust_game_app` crate that assembles a labyrinth scene with simple objectives and HUD feedback.

## Project Layout

```
.
├── assets/                # Scene blueprints, textures, audio, and models
├── crates/
│   ├── engine/            # Reusable engine (ECS, renderer, physics, scripting)
│   └── app/               # Game-specific entry point and custom systems
├── Cargo.toml             # Workspace manifest (builds both crates)
└── target/                # Build artifacts
```

## Features

- **Modern rendering pipeline** using `wgpu`/`winit` with HUD overlays, ray-trace visualization, and per-mesh debug overlays.
- **Entity Component System** (`crates/engine/src/ecs`) providing archetype storage, tag queries, per-component memory reporting, and a global event bus.
- **Physics & interaction** powered by `rapier3d`, now with ray casts exposed through the engine for gameplay queries (shooting, selection) plus automatic collider wireframes to visualize half-extents.
- **Material-aware model loading**: the OBJ loader consumes paired MTL data, resolves diffuse textures per material, and renders each segment with its own atlas (while still allowing manual texture overrides).
- **Shooting gameplay**: left-click fires a physics ray, renders a debug trace, and emits events processed by `ShootingSystem` to destroy tagged targets.
- **Custom systems & scripting** keep gameplay separated from the engine loop; systems can subscribe to events like `ShotEvent` without tight coupling while emitting debug gizmos (boxes, spheres, labels) to visualize logic live.
- **Audio & scene data**: ambient sounds via `rodio`, plus data-driven scenes using `SceneDefinition` builders or YAML assets.
- **Per-entity rotation & scripting**: instanced models now track a quaternion-based `Orientation` component, so scripts can spin or tilt meshes at runtime (the sample target uses a `spinner` script to rotate in place).
+ **Runtime debug controls**: the in-game console can toggle collider lines (`debuglines [on|off|toggle]`) and activate creative mode to inspect the scene freely.

## Building & Running

1. Ensure you have Rust (stable) installed.
2. From the repository root, build and run the bundled app:

   ```bash
   cargo run -p rust_game_app
   ```

3. Controls: use `WASD` to move, mouse to look, and `Esc` to exit.

### Development Tips

- `cargo check` is fast for iterating on shared engine/app changes.
- `cargo test -p rust_game` runs the unit tests that cover ECS/tag behavior.
- Desktops with discrete GPUs give the smoothest experience; if rendering fails, logs in the terminal will include WGPU surface errors.

## Engine Architecture

| Layer | Responsibilities | Key Paths |
| --- | --- | --- |
| Application loop | Configures windowing, input, renderer, physics, audio, scripts, and custom systems. Runs the update/render cycle inside the `winit` event loop. | `crates/engine/src/app.rs` |
| ECS | Entity storage, component management, archetype iteration, tagging, memory reporting. | `crates/engine/src/ecs` |
| Components | Definitions and runtime storage for `Position`, `Orientation`, `Render`, `Model`, `Camera`, `Input`, `Light`, `Texture`, `Terrain`, `Script`, `Physics`. | `crates/engine/src/components` |
| Systems | Runtime behavior for camera input, movement, physics integration, render prep, scripting. | `crates/engine/src/systems` |
| Scene pipeline | Converts `SceneDefinition` data (Rust builders or YAML) into populated ECS entities and scene settings (fog, background gradient, audio). | `crates/engine/src/scene.rs` |
| Rendering | `Renderer` abstraction on top of `wgpu`, handles geometry buffers, textures, lighting, camera matrices, UI text. | `crates/engine/src/rendering` |

### Custom Systems

Custom systems implement the `CustomSystem` trait (see `crates/engine/src/app.rs`). They receive mutable access to the ECS every frame plus a command buffer for scripted actions:

```rust
struct MySystem;

impl CustomSystem for MySystem {
    fn scene_loaded(&mut self, ecs: &mut ECS, scene: &str) { /* ... */ }

    fn update(&mut self, ecs: &mut ECS, scene: &str, commands: &mut Vec<ScriptCommand>) {
        // Read/write ECS data, queue ScriptCommand::LoadScene, ::DebugLine,
        // ::DebugGizmo, ::RemoveComponent, etc.
    }

    fn hud_text(&mut self, ecs: &ECS, scene: &str) -> Option<String> {
        Some("Optional HUD overlay".into())
    }
}
```

Register systems via `GameConfig::with_custom_system`. The sample game installs `ShootingSystem` (`crates/app/src/main.rs`) which listens for `ShotEvent`s, updates HUD text, and removes hit targets.

Need a temporary visualization? Custom systems (and scripts) can push
`ScriptCommand::DebugLine(rust_game::rendering::DebugLine { .. })` into the
provided command buffer to render a colored line for the next frame.

### Debug Console

Press the backtick key (<code>`</code>) during gameplay to toggle a simple
debug console overlay anchored at the bottom of the screen. Type text and press
<kbd>Enter</kbd> to submit a command.
Each submitted line is published as `ConsoleCommandEvent` on the ECS event bus,
so any custom system can react to it:

```rust
for console_event in ecs.drain_events::<ConsoleCommandEvent>() {
    match console_event.text.as_str() {
        "reload" => commands.push(ScriptCommand::LoadScene("labyrinth".into())),
        other => log::info!("Unknown console command: {}", other),
    }
}
```

The latest console history is displayed in the HUD alongside other custom
sections, making it useful for quick debugging cheats or inspection commands.

Built-in helpers:

- `mem` / `memory` — print the current CPU+GPU memory usage gathered from the ECS and renderer.
- `creative` — toggle a zero-gravity flight mode: the player and camera temporarily drop their physics bodies, letting you float freely (Space to rise, Shift to descend) until you run the command again.
- `debuglines` (`on`, `off`, or `toggle`) — control collider wireframe rendering so you can inspect half-extents without recompiling.

### Events & Message Bus

Systems can publish gameplay events for other systems to react to without direct knowledge of each other. Use `ecs.emit_event(MyEvent { ... })` to enqueue an event and `ecs.drain_events::<MyEvent>()` to consume all pending events of that type (consumption clears them for that frame). `CollectibleSystem` emits events that `LabyrinthSystem` listens for to update quest progress, demonstrating how the bus fits into the update loop.

## Game-Specific Logic

`crates/app/src/main.rs` builds a minimal combat sandbox: a player entity with FPS controls, a target with a collider, basic terrain, a sun light, and a textured tree prop imported from `assets/tree.obj`. The target entity now opts into the `spinner` script, which steadily updates its orientation so you can see script-driven rotations in action. `ShootingSystem` consumes shot events to destroy anything tagged `target`, keeping a running HUD tally. The renderer overlays each ray as a debug line so you can see exactly where shots travel, and per-body wireframes make collider tuning straightforward.

Additional YAML scenes in `assets/scene.yml`, `assets/other_scene.yml`, and `assets/labyrinth.yml` showcase a data-driven format for entities that mirrors the Rust builders.

## Extending the Engine

- **Add entities/components**: extend `SceneDefinition` builders (or YAML source) with new entities, tags, and components. `scene::apply_scene_definition` automatically instantiates ECS components from the definition structs.
- **Add scripts**: register script bindings through `GameConfig::with_script_binding` so entities run script logic handled by the scripting system (see `ScriptComponent`).
- **New rendering assets**: drop `.obj` models or textures under `assets/` and reference them from `ModelComponentDefinition`/`TextureComponentDefinition`.
- **Physics tuning**: adjust `PhysicsComponentDefinition` half extents, restitution, and friction to change collision behavior, or set `half_extents` to `None` to auto-fit a box collider to the render/terrain geometry. The physics system syncs Rapier bodies back into ECS positions each frame.
- **HUD/Game state**: implement additional `CustomSystem`s to inject gameplay-specific text or commands; they integrate seamlessly with the engine’s renderer/UI overlay.

## Troubleshooting

- **No window / crash on startup**: ensure your GPU drivers support `wgpu`’s backends; fallback errors are logged via `env_logger`.
- **Audio warnings**: missing or unsupported background audio files simply log a warning and continue without music.
- **Physics desync**: verify dynamic entities have both `InputComponent` (for movement) and `PhysicsComponent` definitions so `PhysicsSystem` can apply forces/velocities.

## License

This repository does not currently declare a license. If you plan to use the engine or assets elsewhere, add an appropriate license file first.
