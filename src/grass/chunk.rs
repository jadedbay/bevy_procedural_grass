use bevy::{prelude::*, utils::HashMap, render::{primitives::{Frustum, Aabb}, extract_component::ExtractComponent}, ecs::query::QueryItem, math::{Vec3A, Affine3A}};

use super::{super::render::instance::GrassInstanceData};

#[derive(Component, Clone)]
pub struct GrassChunks {
    pub chunk_size: f32,
    pub chunks: HashMap<(i32, i32, i32), GrassInstanceData>,
    pub loaded: HashMap<(i32, i32, i32), Handle<GrassInstanceData>>,
    pub render: Vec<Handle<GrassInstanceData>>,
}

impl Default for GrassChunks {
    fn default() -> Self {
        Self {
            chunk_size: 30.,
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
pub struct RenderGrassChunks(pub Vec<Handle<GrassInstanceData>>);

pub fn grass_culling(
    mut query: Query<&mut GrassChunks>,
    camera_query: Query<&Frustum>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for mut chunks in query.iter_mut() {
        for frustum in camera_query.iter() {
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
            
            let mut render_chunks = Vec::new();
            for chunk_coords in chunks_inside {
                if let Some(handle) = chunks.loaded.get(&chunk_coords) {
                    render_chunks.push(handle.clone());
                }
            }

            chunks.render = render_chunks;
        }
    }
}