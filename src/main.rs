
use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, Wireframe}, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, render::{mesh::{VertexAttributeValues, Indices}, render_resource::{Buffer, PrimitiveTopology}, RenderApp, renderer::RenderDevice}};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;

use grass::{grass::{Grass, GrassColor}, GrassPlugin, extract::GrassInstanceData, wind::Wind};
use noise::NoiseFn;

use crate::grass::extract::InstanceData;

pub mod grass;

fn main() {

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        GrassPlugin,
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin,
    ))
    .add_systems(Startup, setup);
    
    app.run();
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
            mesh: meshes.add(grass_mesh()),
            density: 250000,
            ..default()
        },
    ));
     
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 5., -5., 5.)),
        ..default()
    });
}

fn grass_mesh() -> Mesh {
    let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [0.034, 0.0, 0.0],
        [-0.034, 0.0, 0.0],
        [0.032, 0.14, 0.0],
        [-0.032, 0.14, 0.0],
        [0.029, 0.25, 0.0],
        [-0.029, 0.25, 0.0],
        [0.026, 0.34, 0.0],
        [-0.026, 0.34, 0.0],
        [0.023, 0.42, 0.0],
        [-0.023, 0.42, 0.0],
        [0.02, 0.48, 0.0],
        [-0.02, 0.48, 0.0],
        [0.013, 0.55, 0.0],
        [-0.013, 0.55, 0.0],
        [0.0, 0.7, 0.0],
    ]);
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ]);
    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 0.14 / 0.7],
        [0.0, 0.14 / 0.7],
        [0.0, 0.25 / 0.7],
        [1.0, 0.25 / 0.7],
        [0.0, 0.34 / 0.7],
        [1.0, 0.34 / 0.7],
        [0.0, 0.42 / 0.7],
        [1.0, 0.42 / 0.7],
        [0.0, 0.48 / 0.7],
        [1.0, 0.48 / 0.7],
        [0.0, 0.55 / 0.7],
        [1.0, 0.55 / 0.7],
        [0.5, 1.0],
    ]);
    grass_mesh.set_indices(Some(Indices::U32(vec![
        2, 0, 1, 1, 3, 2, 4, 2, 3, 3, 5, 4, 6, 4, 5, 5, 7, 6, 8, 6, 7, 7, 9, 8, 10, 8, 9, 9, 11,  10, 12, 10, 11, 11,13, 12, 14, 12, 13
    ])));

    grass_mesh
}