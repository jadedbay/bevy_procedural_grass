use bevy::{prelude::*, render::mesh::VertexAttributeValues};

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
        let mesh_handle = meshes.add(generate_mesh(terrain));
        commands.entity(entity).insert(mesh_handle);
    }
}

use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use noise::NoiseFn;

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Terrain {
    #[inspector(min = 0, max = 1000)]
    subdivisions: u32,
    #[inspector(min = 0.0001, max = 100.0)]
    height_scale: f32,
    #[inspector(min = 0.0001, max = 100.0)]
    intensity: f32,
    #[inspector(min = 1, max = 1000000000)]
    seed: u32,
    #[inspector()]
    noise: bool,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            subdivisions: 0,
            height_scale: 0.05,
            intensity: 5.,
            seed: 1,
            noise: true,
        }
    }
}

fn update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, Entity, &Terrain), Changed<Terrain>>,
) {
    for (mesh_handle, entity, terrain) in query.iter_mut() {
        let mesh = meshes.add(generate_mesh(terrain));

        commands.entity(entity).insert(mesh);

        meshes.remove(mesh_handle);
    }
}

fn generate_mesh(terrain: &Terrain) -> Mesh {
    let mut mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: terrain.subdivisions });
    if terrain.noise {
        if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            if let VertexAttributeValues::Float32x3(positions) = positions {
                for position in positions.iter_mut() {
                    let y = noise::Perlin::new(terrain.seed).get([(position[0] * terrain.intensity) as f64, (position[2] * terrain.intensity) as f64]) as f32;
                    position[1] += y * terrain.height_scale;
                }
            }
        }
    }
    
    mesh
}