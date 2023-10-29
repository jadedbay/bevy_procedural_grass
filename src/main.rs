
use bevy::{prelude::*, pbr::wireframe::WireframePlugin, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, render::mesh::VertexAttributeValues};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;

use grass::{grass::{Grass, GrassColor}, GrassPlugin, extract::GrassInstanceData, wind::Wind};
use noise::NoiseFn;

pub mod grass;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        GrassPlugin,
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin,
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
    let mut terrain_mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 100 });
    if let Some(positions) = terrain_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let VertexAttributeValues::Float32x3(positions) = positions {
            for position in positions.iter_mut() {
                let y = noise::Perlin::new(1).get([(position[0] * 5.) as f64, (position[2] * 5.) as f64]) as f32;
                position[1] += y * 2.;
            }
        }
    }

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.1, 0.0),
                reflectance: 0.0,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::new(100.0, 1.0, 100.0)),
            ..Default::default()
        }, 
        Grass {
            mesh: asset_server.load::<Mesh, &str>("meshes/grass_blade.glb#Mesh0/Primitive0"),
            density: 24,
            offset_strength: 0.5,
            ..default()
        },
    ));
     
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 5., -5., 5.)),
        ..default()
    });
}