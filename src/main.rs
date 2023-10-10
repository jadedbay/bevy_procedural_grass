use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;

pub mod terrain;
use terrain::{TerrainPlugin, Terrain};

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        TerrainPlugin
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        }, 
        Wireframe,
        Terrain::default()
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..Default::default()
    // });

    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
    //         ..default()
    //     },
    //     FlyCamera::default(),
    // ));
}