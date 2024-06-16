use bevy::prelude::*;
use bevy_procedural_grass::ProceduralGrassPlugin;

fn main() {
    App::new()  
        .add_plugins((
            DefaultPlugins,
            ProceduralGrassPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10., 10.)),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 5., 8.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}