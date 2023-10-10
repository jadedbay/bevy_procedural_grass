use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Terrain>()
            .add_systems(Startup, generate_mesh_on_startup)
            .add_systems(Update, update_mesh); 
    }
}

fn generate_mesh_on_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Terrain), Without<Handle<Mesh>>>,
) {
    for (entity, terrain) in query.iter() {
        let mesh = terrain.create_subdivided_plane();
        let mesh_handle = meshes.add(mesh);
        commands.entity(entity).insert(mesh_handle);
    }
}

use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use image::GenericImageView;

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Terrain {
    #[inspector(min = 1, max = 1000)]
    subdivisions: i32,
    #[inspector(min = 0.0001, max = 100.0)]
    height_scale: f32,
}

impl Terrain {
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

impl Default for Terrain {
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
    mut query: Query<(&Handle<Mesh>, Entity, &Terrain), Changed<Terrain>>,
) {
    for (mesh_handle, entity, terrain) in query.iter_mut() {
        let mesh = terrain.create_subdivided_plane();
        let new_handle = meshes.add(mesh);

        commands.entity(entity).insert(new_handle);

        meshes.remove(mesh_handle);
    }
}
