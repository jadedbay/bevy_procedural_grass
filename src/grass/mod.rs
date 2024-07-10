use bevy::{ecs::query::QueryItem, prelude::*, render::extract_component::ExtractComponent};

pub mod chunk;
pub mod mesh;

use chunk::GrassChunks;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    pub grass: Grass,
    pub mesh: Handle<Mesh>,
}

#[derive(Component, Clone)]
pub struct Grass {
    pub chunk_size: f32,
    pub chunk_count: UVec2,
    pub chunks: GrassChunks,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            chunk_size: 3.0,
            chunk_count: UVec2::new(0, 0),
            chunks: GrassChunks::new(),
        }
    }
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static Handle<Mesh>);
    type QueryFilter = ();
    type Out = (Grass, Handle<Mesh>);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}