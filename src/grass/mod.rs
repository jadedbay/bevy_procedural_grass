use bevy::{ecs::query::QueryItem, prelude::*, render::extract_component::ExtractComponent};

pub mod chunk;
pub mod mesh;

use chunk::GrassChunks;

#[derive(Bundle)]
pub struct GrassBundle {
    pub grass: Grass,
    pub mesh: Handle<Mesh>,
}

#[derive(Component, Clone)]
pub struct Grass {
    pub ground_entity: Entity, 
    pub chunk_size: f32,
    pub chunk_count: UVec2,
    pub chunks: GrassChunks,
}

impl Grass {
    pub fn default(ground_entity: Entity) -> Self {
        Self {
            ground_entity,
            chunk_size: 3.0,
            chunk_count: UVec2::new(0, 0),
            chunks: GrassChunks::new(),
        }
    }
}

impl ExtractComponent for Grass {
    type QueryData = &'static Grass;
    type QueryFilter = ();
    type Out = Grass;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some(item.clone())
    }
}

#[derive(Component, Clone)]
pub struct GrassGround;

impl ExtractComponent for GrassGround {
    type QueryData = &'static Handle<Mesh>;
    type QueryFilter = With<GrassGround>;
    type Out = Handle<Mesh>;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
       Some(item.clone()) 
    }
}