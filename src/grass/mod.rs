use bevy::{ecs::query::QueryItem, math::bounding::{Aabb2d, BoundingVolume}, pbr::MaterialExtension, prelude::*, render::{extract_component::ExtractComponent, render_resource::{AsBindGroup, Buffer, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};

pub mod chunk;
pub mod cull;
pub mod mesh;
pub mod clump;
pub mod config;
pub mod material;

use chunk::{Aabb2dGpu, GrassChunk, GrassChunkBuffers};
use cull::GrassCullChunks;

use crate::{prefix_sum::calculate_workgroup_counts, GrassMaterial};

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

// TODO: rename this i dont like it
#[derive(Component, Clone)]
pub struct GrassGpuInfo {
    pub aabb: Aabb2d,
    pub chunk_size: Vec2,
    pub aabb_buffer: Buffer,
    pub height_scale_buffer: Buffer,
    pub height_offset_buffer: Buffer,

    pub instance_count: usize,
    pub workgroup_count: u32,
    pub scan_workgroup_count: u32,
    pub scan_groups_workgroup_count: u32,
}

impl ExtractComponent for Grass {
    type QueryData = (&'static Grass, &'static GrassGpuInfo, Entity);
    type QueryFilter = ();
    type Out = (Grass, GrassGpuInfo);

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self::Out> {
        Some((item.0.clone(), item.1.clone()))
    }
}

pub(crate) fn grass_setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass, &Parent)>,
    ground_query: Query<&Handle<Mesh>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass, parent) in grass_query.iter() {
        let mesh = meshes.get(ground_query.get(parent.get()).unwrap()).unwrap();
        let mesh_aabb = mesh.compute_aabb().unwrap();
        let mesh_aabb2d = Aabb2d::new(mesh_aabb.center.xz(), mesh_aabb.half_extents.xz());

        let chunk_size = (mesh_aabb2d.max - mesh_aabb2d.min) / (grass.chunk_count.as_vec2());

        let workgroup_count = (((mesh_aabb2d.visible_area() * 0.001) * grass.density) / (grass.chunk_count.x * grass.chunk_count.y) as f32).ceil() as usize;
        
        let instance_count = workgroup_count * 512;

        dbg!(instance_count);
        if instance_count > 256_000 {
            warn!("Instance count: {instance_count}. \nFlickering may occur when instance count is over 256,000.");
        }

        let (scan_workgroup_count, scan_groups_workgroup_count) = calculate_workgroup_counts(instance_count as u32);

        commands.entity(entity).insert((
            GrassGpuInfo {
                aabb: mesh_aabb2d,
                chunk_size,
                aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("aabb_buffer"),
                    contents: bytemuck::cast_slice(&[Aabb2dGpu::from(mesh_aabb2d)]),
                    usage: BufferUsages::UNIFORM,
                }),
                height_scale_buffer: render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: Some("height_scale_buffer"),
                        contents: bytemuck::cast_slice(&[grass.height_map.as_ref().unwrap().scale]),
                        usage: BufferUsages::UNIFORM,
                    }
                ),
                height_offset_buffer: render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: Some("height_offset_buffer"),
                        contents: bytemuck::cast_slice(&[grass.y_offset]),
                        usage: BufferUsages::UNIFORM,
                    }
                ),
                instance_count,
                workgroup_count: workgroup_count as u32,
                scan_groups_workgroup_count,
                scan_workgroup_count,
            },
            GrassCullChunks(HashMap::new()),
        ));
    }
}