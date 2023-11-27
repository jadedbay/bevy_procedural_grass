use bevy::{prelude::*, utils::HashMap, render::{primitives::{Frustum, Aabb}, extract_component::ExtractComponent}, ecs::query::QueryItem, math::{Vec3A, Affine3A}};

use super::{super::render::instance::GrassInstanceData, grass::Grass};

#[derive(Component, Clone)]
pub struct GrassChunks {
    pub chunk_size: f32,
    pub chunks: HashMap<(i32, i32, i32), GrassInstanceData>,
    pub loaded: HashMap<(i32, i32, i32), Handle<GrassInstanceData>>,
}

impl Default for GrassChunks {
    fn default() -> Self {
        Self {
            chunk_size: 12.,
            chunks: HashMap::new(),
            loaded: HashMap::new(),
        }
    }
}

impl GrassChunks {
    pub fn add_data(&mut self, chunk_coords: (i32, i32, i32), instance: GrassInstanceData) {
        self.chunks.insert(chunk_coords, instance);
    }
}

#[derive(Component, Default, Clone)]
pub struct GrassChunkHandles(pub Vec<Handle<GrassInstanceData>>);

impl ExtractComponent for GrassChunkHandles {
    type Query = &'static GrassChunkHandles;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}

pub fn grass_culling(
    mut query: Query<(&Grass, &mut GrassChunks)>,
    mut grass_query: Query<(Entity, &mut GrassChunkHandles)>,
    camera_query: Query<&Frustum>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for (grass, mut chunks) in query.iter_mut() {
        for (entity, mut grass_handles) in grass_query.iter_mut() {
            for frustum in camera_query.iter() {
                if grass.grass_entity == Some(entity) {
                    let aabb = Aabb {
                        center: Vec3A::splat(chunks.chunk_size / 2.),
                        half_extents: Vec3A::splat(chunks.chunk_size / 2.) + Vec3A::new(2., 2., 2.),
                    };
                    
                    let chunk_coords: Vec<(i32, i32, i32)> = chunks.chunks.keys().cloned().collect();
                    
                    let (chunks_inside, chunks_outside): (Vec<_>, Vec<_>) = chunk_coords.iter()
                        .cloned()
                        .partition(|&chunk_coords| {
                            let (x, y, z) = chunk_coords;
                            let world_pos = Affine3A::from_translation(Vec3::new(x as f32, y as f32, z as f32) * chunks.chunk_size);
                            frustum.intersects_obb(&aabb, &world_pos, false, false)
                        });
                
                    for chunk_coords in chunks_outside {
                        chunks.loaded.remove(&chunk_coords);
                    }
                
                    for chunk_coords in chunks_inside.iter() {
                        if !chunks.loaded.contains_key(chunk_coords) {
                            let instance = chunks.chunks.get(chunk_coords).unwrap();
                            let handle = grass_asset.add(instance.clone());
                            chunks.loaded.insert(*chunk_coords, handle);
                        }
                    }
                
                    grass_handles.0.clear();
                    for chunk_coords in chunks_inside {
                        if let Some(handle) = chunks.loaded.get(&chunk_coords) {
                            grass_handles.0.push(handle.clone());
                        }
                    }
                }
            }
        }
    }
}