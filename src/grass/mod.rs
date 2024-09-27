use bevy::{ecs::query::QueryItem, prelude::*, render::{extract_component::ExtractComponent, view::NoFrustumCulling}};

pub mod chunk;
pub mod mesh;
pub mod clump;
pub mod config;

use chunk::{GrassAabb, GrassChunks};

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
    pub chunk_count: UVec2,
    pub density: u32,
    pub height_map: Option<Handle<Image>>,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            ground_entity: None,
            chunk_count: UVec2::splat(0),
            density: 5,
            height_map: None,
        }
    }
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static GrassChunks, &'static GrassAabb);
    type QueryFilter = ();
    type Out = (Grass, GrassChunks, GrassAabb);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone(), item.2.clone()))
    }
}

#[derive(Component, Clone)]
pub struct GrassGround;
