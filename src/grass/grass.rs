use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues, extract_component::ExtractComponent}, utils::HashMap, ecs::query::QueryItem};
#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use bytemuck::{Zeroable, Pod};
use rand::Rng;

use crate::render::instance::{GrassInstanceData, GrassData};

use super::chunk::GrassChunks;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    pub mesh: Handle<Mesh>,
    pub grass: Grass,
    pub grass_chunks: GrassChunks,
    #[bundle()]
    pub spatial: SpatialBundle,
    pub frustum_culling: NoFrustumCulling,
}

pub fn generate_grass(
    mut query: Query<(&Grass, &mut GrassChunks)>,
    mesh_entity_query: Query<(&Transform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (grass, mut chunks) in query.iter_mut() {
        let (transform, mesh_handle) = mesh_entity_query.get(grass.entity.unwrap()).unwrap();
        let mesh = meshes.get(mesh_handle).unwrap();
        chunks.chunks = grass.generate_grass(transform, mesh, chunks.chunk_size);
    }
}

#[derive(Component)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct Grass {
    pub entity: Option<Entity>,
    pub density: u32,
    pub color: GrassColor,
    pub blade: Blade,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            density: 25,
            entity: None,
            color: GrassColor::default(),
            blade: Blade::default(),
        }
    }
}

impl Grass {
    fn generate_grass(&self, transform: &Transform, mesh: &Mesh, chunk_size: f32) -> HashMap<(i32, i32, i32), GrassInstanceData> {
        let mut chunks: HashMap<(i32, i32, i32), GrassInstanceData> = HashMap::new();

        if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                if let Some(indices) = mesh.indices() {
                    let mut triangle = Vec::new();
                    for index in indices.iter() {
                        triangle.push(index);
                        if triangle.len() == 3 {
                            let _result: Vec<GrassData> = {
                                let v0 = Vec3::from(positions[triangle[0] as usize]) * transform.scale;
                                let v1 = Vec3::from(positions[triangle[1] as usize]) * transform.scale;
                                let v2 = Vec3::from(positions[triangle[2] as usize]) * transform.scale;

                                let normal = (v1 - v0).cross(v2 - v0).normalize();
            
                                let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
            
                                let scaled_density = (self.density as f32 * area).ceil() as u32;
            
                                (0..scaled_density).filter_map(|_| {
                                    let mut rng = rand::thread_rng();
            
                                    let r1 = rng.gen::<f32>().sqrt();
                                    let r2 = rng.gen::<f32>();
                                    let barycentric = Vec3::new(1.0 - r1, r1 * (1.0 - r2), r1 * r2);
            
                                    let position = (v0 * barycentric.x + v1 * barycentric.y + v2 * barycentric.z) + transform.translation;
                                
                                    let uv0 = Vec2::from(uvs[triangle[0] as usize]);
                                    let uv1 = Vec2::from(uvs[triangle[1] as usize]);
                                    let uv2 = Vec2::from(uvs[triangle[2] as usize]);
                                    let uv = uv0 * barycentric.x + uv1 * barycentric.y + uv2 * barycentric.z;

                                    let chunk_coords = (
                                        ((position.x) / chunk_size).floor() as i32,
                                        ((position.y) / chunk_size).floor() as i32,
                                        ((position.z) / chunk_size).floor() as i32,
                                    );

                                    let instance = GrassData {
                                        position,
                                        normal,
                                        uv,
                                    };

                                    chunks.entry(chunk_coords).or_insert_with(|| {GrassInstanceData(Vec::new())}).0.push(instance);

                                    None
                                }).collect::<Vec<_>>()
                            };
                            triangle.clear();
                        }
                    }
                }
            }
        }

        chunks
    }
}

impl ExtractComponent for Grass {
    type Query = &'static Grass;
    type Filter = ();
    type Out = (GrassColor, Blade);

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some((item.color.clone(), item.blade.clone()))
    }
}

#[derive(Component, Clone, Copy)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
pub struct GrassColor {
    pub ao: Color,
    pub color_1: Color,
    pub color_2: Color,
}

impl GrassColor {
    pub fn to_array(&self) -> [[f32; 4]; 3] {
        [
            self.ao.into(), 
            self.color_1.into(), 
            self.color_2.into()
        ]
    }
}

impl Default for GrassColor {
    fn default() -> Self {
        Self {
            ao: [0.01, 0.02, 0.05, 1.0].into(),
            color_1: [0.1, 0.23, 0.09, 1.0].into(),
            color_2: [0.12, 0.39, 0.15, 1.0].into(),
        }
    }
}

#[derive(Component, Clone, Copy, Pod, Zeroable)]
#[cfg_attr(feature = "bevy-inspector-egui", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "bevy-inspector-egui", reflect(InspectorOptions))]
#[repr(C)]
pub struct Blade {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub tilt_variance: f32,
    pub flexibility: f32,
    pub curve: f32,
    pub specular: f32,
}

impl Default for Blade {
    fn default() -> Self {
        Self {
            length: 1.5,
            width: 1.,
            tilt: 0.5,
            tilt_variance: 0.2,
            flexibility: 0.5,
            curve: 15.,
            specular: 0.2,
        }
    }
}