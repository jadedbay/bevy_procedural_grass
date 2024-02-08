use bevy::{prelude::*, window::PresentMode, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, render::mesh::VertexAttributeValues, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_procedural_grass::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
            WorldInspectorPlugin::new(),
            ProceduralGrassPlugin::default(),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut terrain_mesh = Mesh::from(shape::Plane { size: 100.0, subdivisions: 100 });
    if let Some(positions) = terrain_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let VertexAttributeValues::Float32x3(positions) = positions {
            for position in positions.iter_mut() {
                let y = noise::Perlin::new(1).get([((position[0]) * 0.05) as f64, ((position[2]) * 0.05) as f64]) as f32;
                position[1] += y;
            }
        }
    }

    let terrain = commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                
                ..default()
            }),
            transform: Transform::from_scale(Vec3::new(1.0, 3.0, 1.0)),
            ..default()
        },
    )).id();

    commands.spawn((
        GrassBundle {
            mesh: meshes.add(GrassMesh::mesh(7)),
            lod: GrassLODMesh::new(meshes.add(GrassMesh::mesh(3))),
            grass: Grass {
                entity: Some(terrain.clone()),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.75, height: 4.0, ..default()})),
            material: materials.add(StandardMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            ..default()
        },
        GrassDisplacer {
            width: 10.,
            height: 30.,
            base_offset: Vec3::new(0., -2., 0.),
        }
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_xyzw(
            -0.4207355,
            -0.4207355,
            0.22984886,
            0.77015114,
        )),
        ..default()
    });
}

fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<GrassDisplacer>>
) {
    for mut transform in query.iter_mut() {
        if input.pressed(KeyCode::W) {
            transform.translation += Vec3::new(0.03, 0.0, 0.0);
        }
        if input.pressed(KeyCode::A) {
            transform.translation += Vec3::new(0.0, 0.0, -0.03);
        }
        if input.pressed(KeyCode::S) {
            transform.translation += Vec3::new(-0.03, 0.0, 0.0);
        }
        if input.pressed(KeyCode::D) {
            transform.translation += Vec3::new(0.0, 0.0, 0.03);
        }
    }
}