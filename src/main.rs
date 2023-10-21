use bevy::{prelude::*, pbr::{wireframe::{WireframePlugin, Wireframe}}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;

use grass::{grass::{CustomMaterialPlugin, Grass, InstanceMaterialData, GrassColorData, GrassColor}, GrassPlugin};
use terrain::{TerrainPlugin, component::Terrain};

pub mod grass;
pub mod terrain;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        TerrainPlugin,
        GrassPlugin,
        CustomMaterialPlugin,
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
        Grass {
            mesh: asset_server.load::<Mesh, &str>("meshes/grass_blade.glb#Mesh0/Primitive0"),
            material_data: InstanceMaterialData::default(),
            density: 5,
            color: GrassColor::default(),
            regenerate: false,
        },
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
}

