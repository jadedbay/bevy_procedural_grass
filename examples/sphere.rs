use bevy::prelude::*;

use bevy_flycam::PlayerPlugin;
use bevy_procedural_grass::{ProceduralGrassPlugin, grass::{mesh::GrassMesh, grass::{GrassBundle, Grass}}}; // imports

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PlayerPlugin,
        ProceduralGrassPlugin::default(), // add procedural grass plugin
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let terrain_mesh = Mesh::try_from(shape::Icosphere { radius: 1.0, subdivisions: 20 }).unwrap();

    let terrain = commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::new(20.0, 20.0, 20.0)),
            ..Default::default()
        },
    )).id();

    // add grass bundle
    commands.spawn(GrassBundle {
        mesh: meshes.add(GrassMesh::mesh()),
        grass: Grass {
            density: 25,
            entity: Some(terrain.clone()), // set entity grass will be placed on (must have a mesh and transform)
            ..default()
        },
        ..default()
    });
     
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        transform: Transform::from_rotation(Quat::from_xyzw(
            -0.4207355,
            -0.4207355,
            0.22984886,
            0.77015114,
        )),
        ..default()
    });
}