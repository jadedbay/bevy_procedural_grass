use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues, extract_component::ExtractComponent, render_resource::{Extent3d, TextureDimension, TextureFormat}}, utils::HashMap, ecs::query::QueryItem};
#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use bytemuck::{Zeroable, Pod};
use rand::Rng;

use crate::render::instance::{GrassChunkData, GrassData};

use super::{chunk::GrassChunks, displacement::GrassDisplacementImage, config::GrassConfig};

#[derive(Bundle, Default)]
pub struct GrassBundle {
    pub mesh: Handle<Mesh>,
    pub lod: GrassLODMesh,
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
    asset_server: Res<AssetServer>,
    config: Res<GrassConfig>,
) {
    for (grass, mut chunks) in query.iter_mut() {
        let (transform, mesh_handle) = mesh_entity_query.get(grass.entity.unwrap()).unwrap();
        let mesh = meshes.get(mesh_handle).unwrap();
        let start = std::time::Instant::now();
        chunks.chunks = grass.generate_grass(transform, mesh, chunks.chunk_size, &asset_server, &config);
        let duration = start.elapsed();
        println!("GRASS GENERATION TIME: {:?}", duration);
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
    fn generate_grass(&self, transform: &Transform, mesh: &Mesh, chunk_size: f32, asset_server: &AssetServer, config: &GrassConfig) -> HashMap<(i32, i32, i32), (GrassChunkData, GrassDisplacementImage)> {
        let mut chunks: HashMap<(i32, i32, i32), (GrassChunkData, GrassDisplacementImage)> = HashMap::new();

        if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
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
                                
                                let chunk_coords = (
                                    (position.x / chunk_size).floor() as i32,
                                    (position.y / chunk_size).floor() as i32,
                                    (position.z / chunk_size).floor() as i32,
                                );

                                let chunk_base = Vec3::new(chunk_coords.0 as f32, chunk_coords.1 as f32, chunk_coords.2 as f32) * chunk_size;
                                let chunk_pos = position - chunk_base;
                                let chunk_uvw = Vec3::new(chunk_pos.x / chunk_size, chunk_pos.y / chunk_size, chunk_pos.z / chunk_size);
                                
                                let instance = GrassData {
                                    position,
                                    normal,
                                    chunk_uvw,
                                };

                                chunks.entry(chunk_coords).or_insert_with(|| {
                                    let image_handle = asset_server.add( 
                                        Image::new_fill(
                                            Extent3d {
                                                width: config.displacement_resolution,
                                                height: config.displacement_resolution,
                                                depth_or_array_layers: 1,
                                            }, 
                                            TextureDimension::D2, 
                                            &[0, 0, 0, 0], 
                                            TextureFormat::Rgba8Unorm,
                                        ));

                                    (GrassChunkData(Vec::new()), GrassDisplacementImage::new(image_handle, config.displacement_resolution))
                                }).0.0.push(instance);

                                None
                            }).collect::<Vec<_>>()
                        };
                        triangle.clear();
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
    pub p1_flexibility: f32,
    pub p2_flexibility: f32,
    pub curve: f32,
    pub specular: f32,
}

impl Default for Blade {
    fn default() -> Self {
        Self {
            length: 1.5,
            width: 0.05,
            tilt: 0.5,
            tilt_variance: 0.2,
            p1_flexibility: 0.5,
            p2_flexibility: 0.5,
            curve: 15.,
            specular: 0.02,
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct GrassLODMesh {
    pub mesh_handle: Option<Handle<Mesh>>,
}

impl GrassLODMesh {
    pub fn new(mesh_handle: Handle<Mesh>) -> Self {
        Self {
            mesh_handle: Some(mesh_handle)
        }
    }
}

impl ExtractComponent for GrassLODMesh {
    type Query = &'static GrassLODMesh;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
}