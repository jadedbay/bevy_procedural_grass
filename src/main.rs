
use std::time::Duration;

use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, Wireframe}, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, render::{mesh::{VertexAttributeValues, Indices}, render_resource::{Buffer, PrimitiveTopology}, RenderApp, renderer::RenderDevice, primitives, texture::{ImageType, CompressedImageFormats, ImageSampler}}, window::PresentMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;

use grass::{grass::Grass, GrassPlugin};
use noise::NoiseFn;

pub mod grass;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }),
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
    mut images: ResMut<Assets<Image>>,
) {
    let mut terrain_mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 100 });
    if let Some(positions) = terrain_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let VertexAttributeValues::Float32x3(positions) = positions {
            for position in positions.iter_mut() {
                let y = noise::Perlin::new(1).get([((position[0]) * 5.) as f64, ((position[2]) * 5.) as f64]) as f32;
                position[1] += y;
            }
        }
    }

    let wind_map_image = Image::from_buffer(
        include_bytes!("grass/assets/images/wind_map2.png"),
        ImageType::Extension("png"),
        CompressedImageFormats::default(),
        false,
        ImageSampler::Default,
    ).unwrap();
    let image_handle = images.add(wind_map_image);

    //let terrain_mesh = Mesh::try_from(shape::Icosphere { radius: 1.0, subdivisions: 20 }).unwrap();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::new(100.0, 2.0, 100.0)),
            ..Default::default()
        }, 
        Grass {
            mesh: meshes.add(grass_mesh()),
            density: 25,
            wind_map_handle: image_handle,
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
        [0.0, 0.63, 0.0],
    ]);

    grass_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 0.14 / 0.63],
        [1.0, 0.14 / 0.63],
        [0.0, 0.25 / 0.63],
        [1.0, 0.25 / 0.63],
        [0.0, 0.34 / 0.63],
        [1.0, 0.34 / 0.63],
        [0.0, 0.42 / 0.63],
        [1.0, 0.42 / 0.63],
        [0.0, 0.48 / 0.63],
        [1.0, 0.48 / 0.63],
        [0.0, 0.55 / 0.63],
        [1.0, 0.55 / 0.63],
        [0.5, 1.0],
    ]);
    grass_mesh.set_indices(Some(Indices::U32(vec![
        2, 0, 1, 1, 3, 2, 4, 2, 3, 3, 5, 4, 6, 4, 5, 5, 7, 6, 8, 6, 7, 7, 9, 8, 10, 8, 9, 9, 11, 10, 12, 10, 11, 11, 13, 12, 14, 12, 13
    ])));

    grass_mesh
}