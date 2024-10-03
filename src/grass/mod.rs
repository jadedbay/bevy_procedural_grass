use bevy::{ecs::query::QueryItem, math::bounding::Aabb2d, pbr::MaterialExtension, prelude::*, render::{extract_component::ExtractComponent, render_resource::{AsBindGroup, Buffer}, view::NoFrustumCulling}};

pub mod chunk;
pub mod mesh;
pub mod clump;
pub mod config;
pub mod material;

use chunk::{GrassChunk, GrassChunkBuffers};

use crate::GrassMaterial;

#[derive(Bundle, Default)]
pub struct GrassBundle {
    pub grass: Grass,
    pub mesh: Handle<Mesh>,
    pub material: Handle<GrassMaterial>,
    #[bundle()]
    pub spatial_bundle: SpatialBundle,
    pub frustum_culling: NoFrustumCulling,
}

#[derive(Reflect, Component, Clone)]
pub struct Grass {
    pub tile_count: UVec2,
    pub chunk_count: UVec2, // TODO: calculate this maybe?
    pub density: f32,
    pub height_map: Option<GrassHeightMap>,
    pub y_offset: f32,
}

#[derive(Reflect, Clone)]
pub struct GrassHeightMap {
    pub map: Handle<Image>,
    pub scale: f32,
}

impl Default for Grass {
    fn default() -> Self {
        Self {
            tile_count: UVec2::splat(1),
            chunk_count: UVec2::splat(1),
            density: 10.0,
            height_map: None,
            y_offset: 0.0,
        }
    }
}

#[derive(Component, Clone)]
pub struct GrassGpuInfo {
    pub aabb: Aabb2d,
    pub aabb_buffer: Buffer,
    pub height_scale_buffer: Buffer,
    pub height_offset_buffer: Buffer,

    pub instance_count: usize,
    pub workgroup_count: u32,
    pub scan_workgroup_count: u32,
    pub scan_groups_workgroup_count: u32,
}

impl ExtractComponent for GrassChunk {
    type QueryData = (&'static GrassChunk, &'static GrassChunkBuffers);
    type QueryFilter = ();
    type Out = (GrassChunk, GrassChunkBuffers);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static GrassGpuInfo, Entity);
    type QueryFilter = ();
    type Out = (Grass, GrassGpuInfo);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}
