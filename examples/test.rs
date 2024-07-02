use bevy::{prelude::*, render::primitives::Aabb};
use bevy_procedural_grass::{grass::{Grass, GrassBundle}, ProceduralGrassPlugin};

fn main() {
    App::new()  
        .add_plugins((
            DefaultPlugins,
            ProceduralGrassPlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy_procedural_grass::util::draw_chunks)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let plane = Plane3d::default().mesh().size(10., 10.).subdivisions(0).build();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(plane),
            ..default()
        },
        GrassBundle::default()
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20., 10., 16.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}