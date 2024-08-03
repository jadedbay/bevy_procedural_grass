use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, view::NoFrustumCulling}};

pub mod chunk;
pub mod mesh;
pub mod clump;
pub mod ground_mesh;
pub mod config;

use chunk::GrassChunks;

use self::chunk::GrassChunksP;

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
    type QueryData = (&'static Grass, &'static GrassChunks, &'static GrassChunksP);
    type QueryFilter = ();
    type Out = (Grass, GrassChunks, GrassChunksP);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone(), item.2.clone()))
    }
}

#[derive(Component, Clone)]
pub struct GrassGround;
