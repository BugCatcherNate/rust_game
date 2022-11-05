use bevy::{prelude::*, render::camera::ScalingMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(cube_movement)
        .run();
}

fn cube_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let cameraSpeed = 5.0;
    for mut transform in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= cameraSpeed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += cameraSpeed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.z -= cameraSpeed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.z += cameraSpeed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            transform.translation.y += cameraSpeed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            transform.translation.y -= cameraSpeed * time.delta_seconds();
        }
    }
}

#[derive(Component)]
struct Cube;

#[derive(Component)]
struct Camera;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Cube);
    // light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            ..default()
        },
        ..default()
    });
    // camera

    commands
        .spawn_bundle(Camera3dBundle {
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::FixedVertical(2.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Camera);
}
