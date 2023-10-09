use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_flycam::*;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
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

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..Default::default()
    // });

    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
    //         ..default()
    //     },
    //     FlyCamera::default(),
    // ));
}

use image::GenericImageView;

fn create_subdivided_plane(subdivisions_x: usize, subdivisions_y: usize) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let img = image::open("assets/heightmap.jpg").unwrap();
    let (width, height) = img.dimensions();
    let width = width - 1;
    let height = height - 1;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    for x in 0..=subdivisions_x {
        for y in 0..=subdivisions_y {
            let x0 = x as f32 / subdivisions_x as f32 - 0.5;
            let y0 = y as f32 / subdivisions_y as f32 - 0.5;

            let height_x = (x as f32 / subdivisions_x as f32 * width as f32) as u32;
            let height_y = (y as f32 / subdivisions_y as f32 * height as f32) as u32;
            let pixel = img.get_pixel(height_x, height_y);
            
            let scale_factor = 0.2; 
            let height_offset = ((pixel[0] as f32 / 255.0) - 0.5) * scale_factor;
    
            positions.push([x0, height_offset, y0]);
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