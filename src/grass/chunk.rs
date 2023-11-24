use bevy::{prelude::*, utils::HashMap, render::{primitives::{Frustum, Aabb}, extract_component::ExtractComponent}, ecs::query::QueryItem, math::{Vec3A, Affine3A}};

use super::{extract::{InstanceData, GrassInstanceData}, grass::{GrassColor, Blade, Grass}};

#[derive(Component, Clone)]
pub struct GrassChunks {
    pub chunk_size: f32,
    pub chunks: HashMap<(i32, i32, i32), GrassInstanceData>,
    pub loaded: HashMap<(i32, i32, i32), Handle<GrassInstanceData>>,
}

#[derive(Component, Default, Clone)]
pub struct GrassToDraw(pub Vec<Handle<GrassInstanceData>>);

impl Default for GrassChunks {
    fn default() -> Self {
        Self {
            chunk_size: 30.,
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

impl ExtractComponent for GrassToDraw {
    type Query = &'static GrassToDraw;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
    }
}

pub fn grass_culling(
    mut query: Query<(&Grass, &mut GrassChunks)>,
    mut grass_query: Query<(Entity, &mut GrassToDraw)>,
    camera_query: Query<&Frustum>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for (grass, mut chunks) in query.iter_mut() {
        for (entity, mut grass_handles) in grass_query.iter_mut() {
            for frustum in camera_query.iter() {
                if grass.grass_entity == Some(entity) {
                    let aabbs: Vec<((i32, i32, i32), Aabb)> = chunks.chunks.iter()
                        .map(|(chunk_coords, _)| {
                            let (x, y, z) = *chunk_coords;
                            let center = Vec3A::new(x as f32, y as f32, z as f32) * chunks.chunk_size + Vec3A::splat(chunks.chunk_size / 2.);
                            let half_extents = Vec3A::splat(chunks.chunk_size / 2.);
                            (*chunk_coords, Aabb { center, half_extents })
                        }).collect();
                
                    let model_to_world = Affine3A::IDENTITY;
                
                    let (chunks_inside, chunks_outside): (Vec<_>, Vec<_>) = aabbs.iter()
                        .partition(|(_, aabb)| frustum.intersects_obb(&aabb, &model_to_world, true, true));
                
                    for (chunk_coords, _) in chunks_outside {
                        chunks.loaded.remove(&chunk_coords);
                    }
                
                    for (chunk_coords, _) in chunks_inside.iter() {
                        if !chunks.loaded.contains_key(chunk_coords) {
                            let instance = chunks.chunks.get(chunk_coords);
                            let handle = grass_asset.add(instance.clone().unwrap().clone());
                            chunks.loaded.insert(*chunk_coords, handle);
                        }
                    }
                
                    grass_handles.0.clear();
                    for (chunk_coords, _) in chunks_inside {
                        if let Some(handle) = chunks.loaded.get(&chunk_coords) {
                            grass_handles.0.push(handle.clone());
                        }
                    }
                }
            }
        }
    }
}