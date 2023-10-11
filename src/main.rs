use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;
use terrain::{TerrainPlugin, Terrain};

pub mod terrain;

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
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        PbrBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        }, 
        Wireframe,
        Terrain::default(),
    ));

    // commands.spawn(PbrBundle {
    //     //mesh: asset_server.load("meshes/grass_blade.glb#Mesh0/Primitive0"),
    //     mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 })),
    //     material: materials.add(Color::GREEN.into()),
    //     ..default()
    // }).insert(Wireframe);
     
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
