use bevy::{prelude::*, render::primitives::Aabb};
use bevy_procedural_grass::{grass::{Grass, GrassBundle}, ProceduralGrassPlugin};

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
    let plane = Plane3d::default().mesh().size(10., 10.).build();
    let aabb = plane.compute_aabb().unwrap();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(plane),
            ..default()
        },
        aabb,
        ShowAabbGizmo {
            color: Some(Color::RED)
        },
        GrassBundle::default()
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 5., 8.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}