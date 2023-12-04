use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use procedural_grass::{ProceduralGrassPlugin, grass::{mesh::GrassMesh, wind::{GrassWind, Wind, WindMap}, grass::{GrassBundle, Grass}}, render::extract::WindData};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PlayerPlugin,
        ProceduralGrassPlugin, // add procedural grass plugin
        WorldInspectorPlugin::new(),
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let terrain_mesh = Mesh::try_from(shape::Icosphere { radius: 1.0, subdivisions: 20 }).unwrap();

    // add global wind resource
    let wind_map = asset_server.add(GrassWind::generate_wind_map(512));
    commands.insert_resource(GrassWind {
        wind_map: wind_map.clone(),
        ..default()
    });

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
            entity: Some(terrain.clone()),
            ..default()
        },
        wind_data: WindData::from(Wind::default()),
        wind_map: WindMap {
            wind_map: wind_map.clone(),
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