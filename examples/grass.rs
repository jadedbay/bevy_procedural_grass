use bevy::prelude::*;
use bevy_procedural_grass::{prelude::*, grass::interactable::GrassInteractable};
use bevy_flycam::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            ProceduralGrassPlugin::default(), // add grass plugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let terrain = commands.spawn(
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                
                ..default()
            }),
            transform: Transform::from_scale(Vec3::new(100.0, 3.0, 100.0)),
            ..default()
        }, 
    ).id();

    // spawn grass
    commands.spawn(GrassBundle {
        mesh: meshes.add(GrassMesh::mesh(7)), // how many segments you want in the mesh (no. of verts = segments * 2 + 1)
        grass: Grass {
            entity: Some(terrain.clone()), // set entity that grass will generate on top of.
            ..default()
        },
        lod: GrassLODMesh::new(meshes.add(GrassMesh::mesh(3))), // optional: enables LOD
        ..default()
    });

    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::WHITE)),
        transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)).with_scale(Vec3::new(1.0, 5.0, 1.0)),
        ..default()
    },GrassInteractable)
    );
     
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