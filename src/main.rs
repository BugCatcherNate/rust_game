use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(cube_movement)
        .run();
}

fn cube_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<&mut Transform, With<Cube>>,
    time: Res<Time>
) {
    for mut transform in head_positions.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= 1.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::D) {

            transform.translation.x += 1.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::W) {

            transform.translation.z -= 1.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::S) {
            
            transform.translation.z += 1.0 * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Space) {
            
            transform.translation.y += 1.0 * time.delta_seconds();
        }
     if keyboard_input.pressed(KeyCode::LShift) {
            
            transform.translation.y -= 1.0 * time.delta_seconds();
        }

    }
}

#[derive(Component)]
struct Cube;


/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
        ..default()
        }, ..default()
    }).insert(Cube);
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {

        ..default()
        },
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}