use std::f64::consts::PI;

use bevy::{color::palettes::css::{RED, WHITE}, math::NormedVectorSpace, pbr::{wireframe::WireframePlugin, DirectionalLightShadowMap}, prelude::*, render::{mesh::VertexAttributeValues, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat}}, window::PresentMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_procedural_grass::{grass::material::create_grass_texture, prelude::*};
use bevy_flycam::prelude::*;
use bevy_compute_noise::prelude::*;

use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use noise::NoiseFn;

fn main() {
    App::new()  
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                },
            ),
            PlayerPlugin,
            ProceduralGrassPlugin::default(),
            WireframePlugin,
            MaterialPlugin::<NormalMaterial>::default(),
            ComputeNoisePlugin::<Perlin2d>::default(),
        ))
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ))
        .add_plugins((
            PerfUiPlugin, 
            WorldInspectorPlugin::default(),
        ))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut normal_materials: ResMut<Assets<NormalMaterial>>,
    mut grass_materials: ResMut<Assets<GrassMaterial>>,
) {
    let mut plane = Plane3d::default().mesh().size(100., 100.).subdivisions(50).build();
    let noise_image = perlin_noise_texture(512, 2.0);
    let wind_image = ComputeNoiseImage::create_image(ComputeNoiseSize::D2(512, 512), ComputeNoiseFormat::Rgba);
    let wind_handle = images.add(wind_image);

    apply_height_map(&mut plane, &noise_image, 0.0);
    plane.compute_normals();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(plane),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            material: materials.add(StandardMaterial {
                base_color: Srgba::rgb(0.5, 0.2, 0.05).into(),
                reflectance: 0.0,
                double_sided: true,
                ..default()
            }),
            visibility: Visibility::Hidden,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            GrassBundle {
                grass: Grass {
                    chunk_count: UVec2::splat(1),
                    density: 25.0,
                    height_map: Some(GrassHeightMap {
                        map: images.add(noise_image),
                        scale: 0.0,
                    }),
                    y_offset: 0.0001,
                },
                mesh: meshes.add(GrassMesh::mesh(7)),
                material: grass_materials.add(
                    GrassMaterial {
                        base: StandardMaterial { 
                            base_color: Srgba::rgb(0.15, 0.24, 0.03).into(),
                            perceptual_roughness: 0.65,
                            reflectance: 0.1,
                            diffuse_transmission: 0.4,
                            double_sided: true,
                            ..default()
                        },
                        extension: GrassMaterialExtension {
                            tip_color: Srgba::rgb(0.15, 0.24, 0.03).into(),
                            width: 0.05,
                            curve: 0.3,
                            tilt: 0.25,
                            midpoint: 0.5,
                            roughness_variance: 0.15,
                            reflectance_variance: 0.1,
                            min_ao: 0.5,
                            midrib_softness: 0.03,
                            rim_position: 0.5,
                            rim_softness: 0.08,
                            width_normal_strength: 0.3,
                            texture_strength: 0.65,
                            texture: Some(images.add(create_grass_texture(2048, 2048, [12.0, 4.0]))),
                            oscillation_speed: 5.0,
                            oscillation_flexibility: 1.0,
                            oscillation_strength: 0.1,
                            wind_direction: Vec2::new(1.0, 1.0).normalize(),
                            wind_speed: 1.0,
                            wind_strength: 1.0,
                            wind_texture: images.add(generate_wind_map(1024, 4.0)),
                        }
                    }
                ),
                spatial_bundle: SpatialBundle {
                    visibility: Visibility::Visible,
                    ..default()
                },
                ..default()
            },
            ComputeNoiseComponent::<Perlin2d> {
                image: wind_handle,
                noise: Perlin2d::new(0, 7, 2, false),
            }
        ));
    });

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Sphere::new(1.0)),
            material: normal_materials.add(NormalMaterial {}),
            transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            ..default()
        },
    ));

    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 5000.0,
    });

    commands
        .spawn(PointLightBundle {
            // transform: Transform::from_xyz(5.0, 8.0, 2.0),
            transform: Transform::from_xyz(1.0, 2.0, 0.0),
            point_light: PointLight {
                intensity: 100_000.0,
                color: RED.into(),
                shadows_enabled: true,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(Sphere::new(0.1).mesh().uv(32, 18)),
                material: materials.add(StandardMaterial {
                    base_color: RED.into(),
                    emissive: LinearRgba::new(4.0, 0.0, 0.0, 0.0),
                    ..default()
                }),
                ..default()
            });
        });


    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::DIRECT_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
        ..default()
    });

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

fn apply_height_map(plane: &mut Mesh, height_map: &Image, height_scale: f32) {
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
                    let noise_value = sample_noise(height_map, uv[0], uv[1]);
                    position[1] += noise_value * height_scale;
                });
        }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct NormalMaterial {}

impl Material for NormalMaterial {
    fn fragment_shader() -> ShaderRef {
        "normal.wgsl".into()
    }
}

pub fn generate_wind_map(size: usize, scale: f64) -> Image {
    let perlin = noise::PerlinSurflet::new(0);

    let mut data = Vec::with_capacity(size * size * 4);

    let (x1, y1, x2, y2) = (-1.0, -1.0, 1.0, 1.0);
    for y in 0..size {
        for x in 0..size {
            let s = x as f64 / size as f64;
            let t = y as f64 / size as f64;
            let dx = x2 - x1;
            let dy = y2 - y1;

            let nx = x1 + (s * 2.0 * PI).cos() * (dx / (2.0 * PI));
            let ny = y1 + (t * 2.0 * PI).cos() * (dy / (2.0 * PI));
            let nz = x1 + (s * 2.0 * PI).sin() * (dx / (2.0 * PI));
            let nw = y1 + (t * 2.0 * PI).sin() * (dy / (2.0 * PI));

            let noise = perlin.get([nx * scale, ny * scale, nz * scale, nw * scale]);
            let noise_scaled = ((noise + 1.0) / 2.0 * 16777215.0) as u32;

            let r = ((noise_scaled >> 16) & 255) as u8;
            let g = ((noise_scaled >> 8) & 255) as u8;
            let b = (noise_scaled & 255) as u8;

            data.push(r); 
            data.push(g); 
            data.push(b); 
            data.push(255);
        }
    }

    Image::new(
        Extent3d {
            width: size as u32, 
            height: size as u32, 
            depth_or_array_layers: 1
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    )
}