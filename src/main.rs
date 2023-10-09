use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_inspector_egui::{quick::{WorldInspectorPlugin, ResourceInspectorPlugin}, prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PanOrbitCameraPlugin,
        ResourceInspectorPlugin::<Terrain>::default()
    ))
    .init_resource::<Terrain>()
    .register_type::<Terrain>() 
    .add_systems(Startup, setup)
    .add_systems(Update, update_mesh)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain: Res<Terrain>,
) {
    let subdivisions = terrain.subdivisions as usize;

    let mesh = create_subdivided_plane(subdivisions, subdivisions);
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
        ..Default::default()
    }).insert(Wireframe);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        //PanOrbitCamera::default(),
    ));

    
}

fn create_subdivided_plane(subdivisions_x: usize, subdivisions_y: usize) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    for x in 0..=subdivisions_x {
        for y in 0..=subdivisions_y {
            let x0 = x as f32 / subdivisions_x as f32 - 0.5;
            let y0 = y as f32 / subdivisions_y as f32 - 0.5;
    
            positions.push([x0, 0.0, y0]);
        }
    }
    
    for x in 0..subdivisions_x {
        for y in 0..subdivisions_y {
            let i = x + y * (subdivisions_x + 1);
    
            indices.push(i as u32);
            indices.push((i + 1) as u32);
            indices.push((i + subdivisions_x + 1) as u32);
    
            indices.push((i + 1) as u32);
            indices.push((i + subdivisions_x + 2) as u32);
            indices.push((i + subdivisions_x + 1) as u32);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    mesh
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Terrain {
    #[inspector(min = 1, max = 1000)]
    subdivisions: i32,
    #[inspector(min = 1.0, max = 1000.0)]
    size: f32,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            subdivisions: 1,
            size: 10.0,
        }
    }
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, &Wireframe, Entity, &mut Transform)>,
    terrain: Res<Terrain>,
) {
    if terrain.is_changed() {
        let subdivisions = terrain.subdivisions as usize;

        for (mesh_handle, _wireframe, entity, mut transform) in query.iter_mut() {
            let mesh = create_subdivided_plane(subdivisions, subdivisions);
            let new_handle = meshes.add(mesh);

            commands.entity(entity).insert(new_handle);
            transform.scale = Vec3::new(terrain.size, terrain.size, terrain.size);

            meshes.remove(mesh_handle);
        }
    }
}