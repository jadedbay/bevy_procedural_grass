use bevy::{pbr::wireframe::{Wireframe, WireframePlugin}, prelude::*, render::{mesh::{SphereKind, VertexAttributeValues}, render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}, texture}, window::PresentMode};
use bevy_procedural_grass::prelude::*;
use bevy_flycam::prelude::*;

use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use noise::NoiseFn;

fn main() {
    App::new()  
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            PlayerPlugin,
            ProceduralGrassPlugin,
            WireframePlugin,
        ))
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ))
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut plane = Plane3d::default().mesh().size(100., 100.).subdivisions(50).build();
    let noise_image = perlin_noise_texture(512, 2.0);

    {
        let (positions, uvs) = plane.attributes_mut().fold(
            (None, None),
            |(positions, uvs), (id, values)| match (id, values) {
                (id, VertexAttributeValues::Float32x3(pos)) if id == Mesh::ATTRIBUTE_POSITION.id => {
                    (Some(pos), uvs)
                }
                (id, VertexAttributeValues::Float32x2(uv)) if id == Mesh::ATTRIBUTE_UV_0.id => {
                    (positions, Some(uv))
                }
                _ => (positions, uvs),
            },
        );

        if let (Some(positions), Some(uvs)) = (positions, uvs) {
            positions
                .iter_mut()
                .zip(uvs.iter())
                .for_each(|(position, uv)| {
                    let noise_value = sample_noise(&noise_image, uv[0], uv[1]);
                    position[1] += noise_value * 6.0;
                });
        }
    }

    let ground = commands.spawn((
        PbrBundle {
            mesh: meshes.add(plane),
            ..default()
        },
       Wireframe,
       GrassGround,
    )).id();

    commands.spawn(
        GrassBundle {
            mesh: meshes.add(GrassMesh::mesh(7)),
            grass: Grass {
                ground_entity: Some(ground),
                chunk_size: 100.0,
                height_map: Some(images.add(noise_image)),
                density: 100,
                ..default()
            },
            ..default()
        }
    );

    commands.spawn(PerfUiBundle::default());
}

fn perlin_noise_texture(texture_size: usize, frequency: f64) -> Image {
    let perlin = noise::Perlin::new(1);

    let mut noise_data = vec![0.0; texture_size * texture_size];
    for y in 0..texture_size {
        for x in 0..texture_size {
            let nx = x as f64 / texture_size as f64;
            let ny = y as f64 / texture_size as f64;
            let noise_value = perlin.get([nx * frequency, ny * frequency]);
            let index = y * texture_size + x;
            noise_data[index] = noise_value as f32;
        }
    }

    Image::new(
        Extent3d {
            width: texture_size as u32,
            height: texture_size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytemuck::cast_slice(&noise_data).to_vec(),
        TextureFormat::R32Float,
        RenderAssetUsages::all(),
    )
}

fn sample_noise(noise_image: &Image, u: f32, v: f32) -> f32 {
    let x = (u * (noise_image.width() - 1) as f32) as u32;
    let y = (v * (noise_image.height() - 1) as f32) as u32;
    let index = (y * noise_image.width() + x) as usize;
    bytemuck::cast_slice(&noise_image.data[index * 4..(index + 1) * 4])[0]
}