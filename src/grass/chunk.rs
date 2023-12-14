use bevy::{prelude::*, utils::HashMap, render::{primitives::{Frustum, Aabb}, extract_component::ExtractComponent}, ecs::query::QueryItem, math::{Vec3A, Affine3A}};

use crate::render::instance::GrassInstanceData;

use super::config::GrassConfig;

#[derive(Clone, Copy)]
pub enum GrassLOD {
    High,
    Low,
}

#[derive(Clone, Copy)]
pub enum CullDimension {
    D2,
    D3,
}


#[derive(Component, Clone)]
pub struct GrassChunks {
    pub chunk_size: f32,
    pub cull_dimension: CullDimension,
    pub chunks: HashMap<(i32, i32, i32), GrassInstanceData>,
    pub loaded: HashMap<(i32, i32, i32), Handle<GrassInstanceData>>,
    pub render: Vec<(GrassLOD, Handle<GrassInstanceData>)>,
}

impl Default for GrassChunks {
    fn default() -> Self {
        Self {
            chunk_size: 30.,
            cull_dimension: CullDimension::D2,
            chunks: HashMap::new(),
            loaded: HashMap::new(),
            render: Vec::new(),
        }
    }
}

impl ExtractComponent for GrassChunks {
    type Query = &'static GrassChunks;
    type Filter = ();
    type Out = RenderGrassChunks;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self::Out> {
        Some(RenderGrassChunks(item.render.clone()))
    }
}

#[derive(Component, Default, Clone)]
pub struct RenderGrassChunks(pub Vec<(GrassLOD, Handle<GrassInstanceData>)>);

pub(crate) fn grass_culling(
    mut query: Query<&mut GrassChunks>,
    camera_query: Query<(&Transform, &Frustum)>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
    grass_config: Res<GrassConfig>,
) {
    for mut chunks in query.iter_mut() {
        chunks.render.clear();
        
        for (transform, frustum) in camera_query.iter() {
            let aabb = Aabb {
                center: Vec3A::splat(chunks.chunk_size / 2.),
                half_extents: Vec3A::splat(chunks.chunk_size / 2.) + Vec3A::new(2., 2., 2.),
            };
            
            let chunk_coords: Vec<(i32, i32, i32)> = chunks.chunks.keys().cloned().collect();
            
            let mut chunks_inside = Vec::new();
            let mut chunks_outside = Vec::new();

            for chunk_coords in chunk_coords.iter().cloned() {
                let (x, y, z) = chunk_coords;
                let world_pos = Vec3::new(x as f32, y as f32, z as f32) * chunks.chunk_size;
                
                let d3_distance = (world_pos + Vec3::from(aabb.center) - transform.translation).length();

                let lod_type = match d3_distance <= grass_config.lod_distance {
                    true => GrassLOD::High,
                    false => GrassLOD::Low,
                };

                let cull_distance = match chunks.cull_dimension {
                    CullDimension::D2 => ((world_pos.xz() + aabb.center.xz()) - transform.translation.xz()).length(),
                    CullDimension::D3 => d3_distance,
                };
                
                if frustum.intersects_obb(&aabb, &Affine3A::from_translation(world_pos), false, false) && cull_distance <= grass_config.cull_distance {
                    chunks_inside.push((chunk_coords, lod_type));
                } else {
                    chunks_outside.push(chunk_coords);
                }
            }
        
            for chunk_coords in chunks_outside {
                chunks.loaded.remove(&chunk_coords);
            }
        
            for chunk_coords in chunks_inside.iter() {
                if !chunks.loaded.contains_key(&chunk_coords.0) {
                    let instance = chunks.chunks.get(&chunk_coords.0).unwrap();
                    let handle = grass_asset.add(instance.clone());
                    chunks.loaded.insert(chunk_coords.0, handle);
                }
            }
            
            let mut render_chunks = Vec::new();
            for chunk_coords in chunks_inside {
                if let Some(handle) = chunks.loaded.get(&chunk_coords.0) {
                    render_chunks.push((chunk_coords.1, handle.clone()));
                }
            }

            chunks.render.extend(render_chunks);
        }
    }
}