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

- **Modern rendering pipeline** using `wgpu` with `winit` for cross-platform windowing/input and HUD text via the renderer’s UI overlay.
- **Entity Component System** purpose-built for this demo (`crates/engine/src/ecs`) providing archetype storage, tag queries, and per-component memory breakdown.
- **Physics & movement** powered by `rapier3d` with dynamic, kinematic, and static bodies synced into the ECS positions.
- **Lighting model** supporting one directional light plus multiple point lights for emissive props like the patrol orbs.
- **Automatic collider sizing**: omit `half_extents` in a `PhysicsComponentDefinition` to derive an axis-aligned box from the attached render/terrain geometry.
- **Audio** playback through `rodio`, supporting ambient scene sounds that loop in the background.
- **Script hooks and custom systems**: the engine exposes a `CustomSystem` trait plus script bindings so gameplay logic can extend the core update loop without modifying the engine.
- **Data-driven scenes** assembled through `SceneDefinition`/`SceneLibrary` (Rust) or the YAML helpers under `assets/`, showing how to go from authored data to runtime ECS entities.

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
| Components | Definitions and runtime storage for `Position`, `Render`, `Model`, `Camera`, `Input`, `Light`, `Texture`, `Terrain`, `Script`, `Physics`. | `crates/engine/src/components` |
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
        // Read/write ECS data, queue ScriptCommand::LoadScene or ::RemoveComponent, etc.
    }

    fn hud_text(&mut self, ecs: &ECS, scene: &str) -> Option<String> {
        Some("Optional HUD overlay".into())
    }
}
```

Register systems via `GameConfig::with_custom_system`. The sample game installs `CollectibleSystem` and `LabyrinthSystem` (`crates/app/src/main.rs`) to drive quest logic.

## Game-Specific Logic

`crates/app/src/main.rs` builds the labyrinth scene in Rust code. Highlights:

- The `Explorer` entity is tagged with `player` and `camera`, has FPS-style camera/input components, and a dynamic physics body.
- Terrain, exit obelisk, hazard crystal, patrol orbs, lighting, and walls are composed through helper functions (`cube_with_texture`, `directional_light`, etc.).
- `CollectibleSystem` tracks tagged collectibles and removes them from the ECS once the player approaches them, updating HUD text.
- `LabyrinthSystem` monitors key entities, unlocks the exit after all are collected, and displays progress/mission status on the HUD.

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
