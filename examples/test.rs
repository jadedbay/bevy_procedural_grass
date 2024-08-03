use bevy::{pbr::wireframe::{Wireframe, WireframePlugin}, prelude::*, render::mesh::{SphereKind, VertexAttributeValues}, window::PresentMode};
use bevy_procedural_grass::{prelude::*, util::draw_chunks};
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
            // WireframePlugin,
        ))
        .add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ))
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_chunks)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut plane = Plane3d::default().mesh().size(10., 10.).subdivisions(1).build();
    // if let Some(positions) = plane.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
    //     if let VertexAttributeValues::Float32x3(positions) = positions {
    //         for position in positions.iter_mut() {
    //             let y = noise::Perlin::new(1).get([(position[0] * 0.2) as f64, (position[2] * 0.2) as f64]) as f32;
    //             position[1] += y;
    //         }
    //     }
    // }

    let sphere = Sphere::new(5.0).mesh().kind(SphereKind::Ico { subdivisions: 2 }).build();

    let ground = commands.spawn((
        PbrBundle {
            mesh: meshes.add(plane),
            ..default()
        },
       GrassGround,
    //    Wireframe,
    )).id();

    commands.spawn(
        GrassBundle {
            mesh: meshes.add(GrassMesh::mesh(7)),
            grass: Grass {
                ground_entity: Some(ground),
                chunk_size: 100.0,
                ..default()
            },
            ..default()
        }
    );

    commands.spawn(PerfUiBundle::default());
}
