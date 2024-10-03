
use bevy::{math::bounding::{Aabb2d, BoundingVolume}, prelude::*, render::{render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, ShaderType}, renderer::RenderDevice, view::NoFrustumCulling}, utils::HashMap};
use super::Grass;
use crate::{grass::{cull::GrassCullChunks, GrassGpuInfo}, prefix_sum::{calculate_workgroup_counts, PrefixSumBuffers}, render::instance::GrassInstanceData, GrassMaterial};

#[derive(Component, Clone)]
pub struct GrassChunk {
    pub grass_entity: Entity,
    pub aabb: Aabb2d,

    pub instance_count: usize,
    pub scan_workgroup_count: u32,
}

#[derive(Component, Clone)]
pub struct GrassChunkBuffers {
    pub aabb_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub vote_buffer: Buffer,
    pub compact_buffer: Buffer,
    pub indirect_args_buffer: Buffer,
    pub(crate) prefix_sum_buffers: PrefixSumBuffers,
}

impl GrassChunkBuffers {
    pub(crate) fn create_buffers(
        render_device: &RenderDevice,
        aabb: Aabb2d,
        instance_count: usize,
        scan_workgroup_count: u32,
    ) -> Self {
        Self {
            aabb_buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("aabb_buffer"),
                contents: bytemuck::cast_slice(&[Aabb2dGpu::from(aabb)]),
                usage: BufferUsages::UNIFORM,
            }),
            instance_buffer: render_device.create_buffer(&BufferDescriptor {
                label: Some("instance_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
                mapped_at_creation: false, 
            }),
            vote_buffer: render_device.create_buffer(&BufferDescriptor {
                label: Some("vote_buffer"),
                size: (std::mem::size_of::<u32>() * instance_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            compact_buffer: render_device.create_buffer(&BufferDescriptor {
                label: Some("compact_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * instance_count) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            indirect_args_buffer: render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indirect_indexed_args"),
                    contents: DrawIndexedIndirectArgs {
                    index_count: 39, // TODO
                    instance_count: 0,
                    first_index: 0,
                    base_vertex: 0,
                    first_instance: 0,
                }.as_bytes(),
                usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            }),
            prefix_sum_buffers: PrefixSumBuffers::create_buffers(
                &render_device, 
                instance_count as u32, 
                scan_workgroup_count,
            )
        }
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

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub(crate) struct Aabb2dGpu {
    min: Vec2,
    max: Vec2,
}

impl From<Aabb2d> for Aabb2dGpu {
    fn from(aabb: Aabb2d) -> Self {
        Self {
            min: aabb.min,
            max: aabb.max,
        }
    }
}
