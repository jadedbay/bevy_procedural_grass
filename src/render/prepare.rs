use bevy::{
    ecs::world, gizmos::aabb, prelude::*, render::{
        render_resource::{
            BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs, PipelineCache, SpecializedComputePipelines
        },
        renderer::RenderDevice,
        view::ViewUniforms,
    }, utils::HashMap
};

use super::{
    instance::GrassInstanceData,
    pipeline::GrassComputePipeline,
};
use crate::{grass::{
    chunk::{BoundingBox, GrassChunks, GrassChunksP},
    ground_mesh::GroundMesh,
}, prefix_sum::{create_prefix_sum_bind_group_buffers, PrefixSumBindGroup, PrefixSumPipeline}};

#[derive(Resource, Default)]
pub struct GrassEntities(pub HashMap<Entity, (GrassStage, Vec<Buffer>)>);

#[derive(Clone, Copy, Default, PartialEq)]
pub enum GrassStage {
    #[default]
    Loading,
    Compute, // TODO pre/post readback
    Cull,
}

#[derive(Clone)]
pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub indirect_args_buffer: Buffer,

    pub cull_bind_group: BindGroup,

    pub cull_workgroup_count: u32,
    pub workgroup_count: u32,
    pub compact_workgroup_count: u32,

    pub compact_buffer: Buffer,
    pub compact_bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct GrassBufferBindGroup {
    pub mesh_bind_group: BindGroup,
    pub triangle_count: u32,
    pub chunks: Vec<GrassChunkBufferBindGroup>,
    pub prefix_sum_chunks: Vec<PrefixSumBindGroup>,
}

pub(crate) fn prepare_grass_bind_groups(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    prefix_sum_pipeline: Res<PrefixSumPipeline>,
    mut grass_entities: ResMut<GrassEntities>,
    query: Query<(Entity, &GrassChunks, &GroundMesh, &GrassChunksP)>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let chunk_layout = pipeline.chunk_layout.clone();
    let cull_layout = pipeline.cull_layout.clone();
    let compact_layout = pipeline.compact_layout.clone();

    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (entity, chunks, ground_mesh, chunksp) in query.iter() {
        let mesh_bind_group = render_device.create_bind_group(

            Some("mesh_position_bind_group"),
            &mesh_layout,
            &BindGroupEntries::sequential((
                ground_mesh.positions_buffer.as_entire_binding(),
                ground_mesh.indices_buffer.as_entire_binding(),
            )),
        );

        let mut chunk_bind_groups = Vec::new();
        let mut prefix_sum_bind_groups = Vec::new();

        let (buffers_created, mut instance_buffers) = if let Some((_, buffers)) = grass_entities.0.get(&entity) {
            (true, buffers.clone())
        } else {
            (false, Vec::new())
        };

        for (i, (_, chunk)) in chunks.0.clone().into_iter().enumerate() {
            let aabb_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("aabb_buffer"),
                contents: bytemuck::cast_slice(&[BoundingBox::from(chunk.aabb)]),
                usage: BufferUsages::UNIFORM,
            });

            let indices_index_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_index_buffer"),
                    contents: bytemuck::cast_slice(chunk.indices_index.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            // let indices_index_buffer = render_device.create_buffer(&BufferDescriptor {
            //     label: Some("indices_index"),
            //     size: (std::mem::size_of::<u32>() * chunksp.triangle_count * chunksp.chunk_count() as usize)
            //         as u64,
            //     usage: BufferUsages::STORAGE,
            //     mapped_at_creation: false,
            // });

            // let counts_buffer = render_device.create_buffer(&BufferDescriptor {
            //     label: Some("counts_buffer"),
            //     size: (std::mem::size_of::<u32>() * 4) as u64,
            //     usage: BufferUsages::STORAGE,
            //     mapped_at_creation: false,
            // });

            let vote_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("vote_buffer"),
                size: (std::mem::size_of::<u32>() * chunk.instance_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            if !buffers_created {
                instance_buffers.push(render_device.create_buffer(&BufferDescriptor {
                    label: Some("grass_data_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * chunk.instance_count) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }))
            }

            let chunk_bind_group = render_device.create_bind_group(
                Some("chunk_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    aabb_buffer.as_entire_binding(),
                    indices_index_buffer.as_entire_binding(),
                    instance_buffers[i].as_entire_binding(),
                    view_uniform.clone(),
                )),
            );

            let compact_buffer = render_device.create_buffer(&BufferDescriptor {
                label: Some("compact_buffer"),
                size: (std::mem::size_of::<GrassInstanceData>() * chunk.instance_count) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                mapped_at_creation: false,
            });

            let indirect_indexed_args_buffer =
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("indirect_indexed_args"),
                    contents: DrawIndexedIndirectArgs {
                        index_count: 39, // TODO
                        instance_count: 0,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: 0,
                    }
                    .as_bytes(),
                    usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
                });
            
            let prefix_sum_bind_group = create_prefix_sum_bind_group_buffers(
                &render_device,
                &prefix_sum_pipeline,
                &vote_buffer,
                chunk.instance_count as u32,
                chunk.scan_workgroup_count,
                chunk.scan_groups_workgroup_count,
            );

            let cull_bind_group = render_device.create_bind_group(
                Some("cull_bind_group"),
                &cull_layout,
                &BindGroupEntries::sequential((
                    aabb_buffer.as_entire_binding(),
                    instance_buffers[i].as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    view_uniform.clone(),
                ))
            );

            let compact_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &compact_layout,
                &BindGroupEntries::sequential((
                    instance_buffers[i].as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    prefix_sum_bind_group.scan_buffer.as_entire_binding(),
                    prefix_sum_bind_group.scan_blocks_out_buffer.as_entire_binding(),
                    compact_buffer.as_entire_binding(),
                    indirect_indexed_args_buffer.as_entire_binding(),
                )),
            );

            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                indirect_args_buffer: indirect_indexed_args_buffer,

                cull_bind_group,

                cull_workgroup_count: (chunk.instance_count as f32 / 256.).ceil() as u32,
                workgroup_count: chunk.workgroup_count,
                compact_workgroup_count: chunk.scan_workgroup_count,

                compact_buffer,
                compact_bind_group,
            });

            prefix_sum_bind_groups.push(prefix_sum_bind_group);
        }

        let buffer_bind_group = GrassBufferBindGroup {
            mesh_bind_group,
            triangle_count: chunksp.triangle_count as u32,
            chunks: chunk_bind_groups,
            prefix_sum_chunks: prefix_sum_bind_groups,
        };
 

        commands.entity(entity).insert(buffer_bind_group.clone());
        let _ = grass_entities.0.try_insert(entity, (GrassStage::default(), instance_buffers));
    }
}
