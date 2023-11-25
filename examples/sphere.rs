use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use procedural_grass::{ProceduralGrassPlugin, grass::{grass::Grass, mesh::GrassMesh}};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PlayerPlugin,
        ProceduralGrassPlugin,
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

    let wind_map  = asset_server.load("images/wind_map.png");

    commands.spawn((
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
        Grass {
            mesh: meshes.add(GrassMesh::mesh()),
            density: 25,
            wind_map_handle: wind_map,
            ..default()
        },
    ));
     
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