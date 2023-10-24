use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};
use noise::NoiseFn;

#[derive(Reflect, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct PerlinNoise {
    pub seed: u32,
    #[inspector(min = 0.0001, max = 100.0)]
    pub intensity: f32,
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self {
            seed: 1,
            intensity: 5.,
        }
    }
}

#[derive(Reflect, Component, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Terrain {
    #[inspector(min = 0, max = 1000)]
    subdivisions: u32,
    #[inspector(min = 0.0001, max = 100.0)]
    height_scale: f32,
    pub noise: Option<PerlinNoise>
}

impl Terrain {
    pub fn get_subdivisions(&self) -> u32 {
        self.subdivisions
    }

    pub fn get_height_scale(&self) -> f32 {
        self.height_scale
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            subdivisions: 100,
            height_scale: 2.,
            noise: Some(PerlinNoise::default())
        }
    }
}

pub fn generate_mesh_on_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Terrain), Without<Handle<Mesh>>>,
) {
    for (entity, terrain) in query.iter() {
        let mesh_handle = meshes.add(generate_mesh(terrain));
        commands.entity(entity).insert(mesh_handle);
    }
}

pub fn update_mesh(
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
    if let Some(noise) = &terrain.noise {
        if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            if let VertexAttributeValues::Float32x3(positions) = positions {
                for position in positions.iter_mut() {
                    let y = noise::Perlin::new(noise.seed).get([(position[0] * noise.intensity) as f64, (position[2] * noise.intensity) as f64]) as f32;
                    position[1] += y * terrain.height_scale;
                }
            }
        }
    }
    
    mesh
}