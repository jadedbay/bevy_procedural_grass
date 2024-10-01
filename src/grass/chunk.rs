
use bevy::{math::{bounding::{Aabb2d, BoundingVolume}, Affine3A}, prelude::*, render::{primitives::{Aabb, Frustum}, render_resource::{Buffer, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, ShaderType}, renderer::RenderDevice, view::NoFrustumCulling}};
use super::{config::GrassConfig, Grass};
use crate::{grass::GrassGpuInfo, prefix_sum::{calculate_workgroup_counts, PrefixSumBuffers}, render::instance::GrassInstanceData};

#[derive(Component, Clone)]
pub struct GrassChunk {
    pub grass_entity: Entity,
    pub aabb: Aabb2d,

    instance_count: usize,
    scan_workgroup_count: u32,
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
    fn create_buffers(
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

pub(crate) fn create_chunks(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grass_query: Query<(Entity, &Grass, &Handle<Mesh>, &Parent)>,
    ground_query: Query<&Handle<Mesh>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass, mesh_handle, parent) in grass_query.iter() {
        let mut chunks = Vec::new();

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

        for x in 0..grass.chunk_count.x { 
            for z in 0..grass.chunk_count.y {
                let chunk_min = mesh_aabb2d.min + Vec2::new(x as f32, z as f32) * chunk_size;
                let chunk_max = chunk_min + chunk_size; 
                let aabb = Aabb2d {
                    min: chunk_min,
                    max: chunk_max,
                };

                let chunk = commands.spawn(
                    (
                        GrassChunk {
                            grass_entity: entity,
                            aabb,
                            instance_count,
                            scan_workgroup_count,
                        },
                        mesh_handle.clone(),
                        SpatialBundle::default(),
                        NoFrustumCulling,
                    )
                ).id();

                chunks.push(chunk);
            }
         }
        commands.entity(entity).push_children(chunks.as_slice());
        commands.entity(entity).insert(
            GrassGpuInfo {
                aabb: mesh_aabb2d,
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
        );
    }
}

pub(crate) fn cull_chunks(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut query: Query<(Entity, &GrassChunk, &Parent)>,
    q_chunk_buffers: Query<&GrassChunkBuffers>, 
    grass_query: Query<&Grass>,
    camera_query: Query<(&Transform, &Frustum)>,
    grass_config: Res<GrassConfig>,
) {
    'chunk: for (entity, chunk, parent) in query.iter_mut() {
        let grass = grass_query.get(parent.get()).unwrap();

        let aabb = Aabb::from_min_max(
            Vec3::new(chunk.aabb.min.x, -grass.height_map.as_ref().unwrap().scale, chunk.aabb.min.y),
            Vec3::new(chunk.aabb.max.x, grass.height_map.as_ref().unwrap().scale, chunk.aabb.max.y),
        );

        for (transform, frustum) in camera_query.iter() {
            if (chunk.aabb.center() - transform.translation.xz()).length() < grass_config.cull_distance 
            && frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, false, false) {
                match q_chunk_buffers.get(entity) {
                    Ok(_) => {}
                    Err(_) => {
                        commands.entity(entity).insert(GrassChunkBuffers::create_buffers(
                            &render_device,
                            chunk.aabb,
                            chunk.instance_count,
                            chunk.scan_workgroup_count,
                        )); 
                    }
                }
                continue 'chunk;
            } else {
                match q_chunk_buffers.get(entity) {
                    Ok(_) => {
                        commands.entity(entity).remove::<GrassChunkBuffers>();
                    }
                    Err(_) => {}
                }
            }
        }
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
