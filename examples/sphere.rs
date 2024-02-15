use bevy::prelude::*;
use bevy_procedural_grass::prelude::*;
use bevy_flycam::PlayerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PlayerPlugin,
        ProceduralGrassPlugin::default(), // add procedural grass plugin
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let terrain_mesh = Mesh::try_from(shape::Icosphere { radius: 1.0, subdivisions: 20 }).unwrap();

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
        mesh: meshes.add(GrassMesh::mesh(7)),
        grass: Grass {
            density: 25,
            entity: Some(terrain.clone()), // set entity grass will be placed on (must have a mesh and transform)
            ..default()
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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.75, height: 4.0, ..default()})),
            material: materials.add(StandardMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            ..default()
        },
        GrassDisplacer {
            width: 15.,
            base_offset: Vec3::new(0., -2., 0.),
        }
    ));
}