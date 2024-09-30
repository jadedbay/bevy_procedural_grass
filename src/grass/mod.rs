use bevy::{ecs::query::QueryItem, math::bounding::Aabb2d, prelude::*, render::{extract_component::ExtractComponent, render_resource::Buffer, view::NoFrustumCulling}};

pub mod chunk;
pub mod mesh;
pub mod clump;
pub mod config;

use chunk::{GrassChunk, GrassChunks};

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
    pub height_map: Option<GrassHeightMap>,
    pub y_offset: f32,
}

#[derive(Clone)]
pub struct GrassHeightMap {
    pub map: Handle<Image>,
    pub scale: f32,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            ground_entity: None,
            chunk_count: UVec2::splat(0),
            density: 5,
            height_map: None,
            y_offset: 0.0,
        }
    }
}

#[derive(Component, Clone)]
pub struct GrassGpuInfo {
    pub aabb: Aabb2d,

    pub instance_count: usize,
    pub workgroup_count: u32,
    pub scan_workgroup_count: u32,
    pub scan_groups_workgroup_count: u32,
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static GrassChunk, &'static GrassGpuInfo, &'static Visibility);
    type QueryFilter = ();
    type Out = (Grass, GrassChunk, GrassGpuInfo);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        if item.3 == Visibility::Hidden { return None; }
        Some((item.0.clone(), item.1.clone(), item.2.clone()))
    }
}

#[derive(Component, Clone)]
pub struct GrassGround;