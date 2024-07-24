use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, view::NoFrustumCulling}};

pub mod chunk;
pub mod mesh;
pub mod clump;
pub mod ground_mesh;

use chunk::GrassChunks;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    pub grass: Grass,
    pub mesh: Handle<Mesh>,
    #[bundle()]
    pub spatial_bundle: SpatialBundle,
    pub frustum_culling: NoFrustumCulling,
}

#[derive(Component, Clone)]
pub struct Grass {
    pub ground_entity: Option<Entity>, 
    pub chunk_size: f32,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            ground_entity: None,
            chunk_size: 30.0,
        }
    }
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static GrassChunks);
    type QueryFilter = ();
    type Out = (Grass, GrassChunks);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}

#[derive(Component, Clone)]
pub struct GrassGround;