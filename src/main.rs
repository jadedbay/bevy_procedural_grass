use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, pbr::wireframe::{WireframePlugin, Wireframe}};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, InspectorOptions, prelude::ReflectInspectorOptions};
use bevy_flycam::*;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new()
    ))
    .register_type::<TerrainMesh>()
    .add_systems(Startup, setup)
    .add_systems(Update, update_mesh)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain = TerrainMesh::default();
    let mesh = terrain.create_subdivided_plane();
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        }, 
        Wireframe,
        terrain
    ));

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

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
struct TerrainMesh {
    #[inspector(min = 1, max = 1000)]
    subdivisions: i32,
    #[inspector(min = 0.0001, max = 100.0)]
    height_scale: f32,
}

impl TerrainMesh {
    fn create_subdivided_plane(&self) -> Mesh {
        let subdivisions_x = self.subdivisions as usize;
        let subdivisions_y = self.subdivisions as usize;
        let scale_factor = self.height_scale;
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
                
                let height_offset = (pixel[0] as f32 / 255.0) * scale_factor;
        
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
}

impl Default for TerrainMesh {
    fn default() -> Self {
        Self {
            subdivisions: 1,
            height_scale: 0.2,
        }
    }
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, Entity, &TerrainMesh), Changed<TerrainMesh>>,
) {
    for (mesh_handle, entity, terrain) in query.iter_mut() {
        let mesh = terrain.create_subdivided_plane();
        let new_handle = meshes.add(mesh);

        commands.entity(entity).insert(new_handle);

        meshes.remove(mesh_handle);
    }
}