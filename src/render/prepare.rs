use bevy::{prelude::*, render::{render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs}, renderer::RenderDevice, view::ViewUniforms}};

use crate::grass::{chunk::{BoundingBox, GrassChunks}, ground_mesh::GroundMesh};
use super::{instance::GrassInstanceData, pipeline::{GrassComputePPSPipelines, GrassComputePipeline}};

#[derive(Clone)]
pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub indirect_buffer: Buffer,

    pub workgroup_count: usize,
    pub scan_workgroup_count: usize,
    pub scan_blocks_workgroup_count: usize,

    pub compact_buffer: Buffer,
    pub scan_bind_group: BindGroup,
    pub scan_blocks_bind_group: BindGroup,
    pub compact_bind_group: BindGroup,
}

#[derive(Clone)]
pub struct GrassBindGroupsP {
    pub chunk_bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct GrassBufferBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunks: Vec<GrassChunkBufferBindGroup>,
    // pub chunksp: Vec<>
}

pub(crate) fn prepare_grass_bind_groups(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    sps_pipelines: Res<GrassComputePPSPipelines>,
    query: Query<(Entity, &GrassChunks, &GroundMesh)>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let chunk_layout = pipeline.chunk_layout.clone();

    let scan_layout = sps_pipelines.scan_layout.clone();
    let scan_blocks_layout = sps_pipelines.scan_blocks_layout.clone();
    let compact_layout = sps_pipelines.compact_layout.clone();

    let Some(view_uniform) = view_uniforms.uniforms.binding() else {
        return;
    };

    for (entity, chunks, ground_mesh) in query.iter() {        
        let mesh_positions_bind_group = render_device.create_bind_group(
            Some("mesh_position_bind_group"),
            &mesh_layout,
            &BindGroupEntries::sequential((
                ground_mesh.positions_buffer.as_entire_binding(),
                ground_mesh.indices_buffer.as_entire_binding(),
            ))
        );

        let mut chunk_bind_groups = Vec::new();

        for (_, chunk) in chunks.0.clone() {

            let aabb_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("aabb_buffer"),
                    contents: bytemuck::cast_slice(&[BoundingBox::from(chunk.aabb)]),
                    usage: BufferUsages::UNIFORM,
                }
            );

            let indices_index_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_index_buffer"),
                    contents: bytemuck::cast_slice(chunk.indices_index.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            let vote_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("vote_buffer"),
                    size: (std::mem::size_of::<u32>() * chunk.instance_count) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let instance_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("grass_data_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * chunk.instance_count) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE, 
                    mapped_at_creation: false,
                }
            );

            let chunk_bind_group = render_device.create_bind_group(
                Some("chunk_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    aabb_buffer.as_entire_binding(),
                    indices_index_buffer.as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    instance_buffer.as_entire_binding(),
                    view_uniform.clone(),
                ))
            );

            let scan_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("scan_output_buffer"),
                    size: (std::mem::size_of::<u32>() * chunk.instance_count) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }  
            );
            let scan_blocks_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("scan_output_buffer"),
                    size: (std::mem::size_of::<u32>() * chunk.scan_workgroup_count) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }  
            );

            let scan_blocks_out_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("scan_output_buffer"),
                    size: (std::mem::size_of::<u32>() * chunk.scan_workgroup_count) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }  
            );

            let compact_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("compact_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * chunk.instance_count) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let indirect_indexed_args_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
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
                }
            );
            
            let scan_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &scan_layout,
                &BindGroupEntries::sequential((
                    vote_buffer.as_entire_binding(),
                    scan_buffer.as_entire_binding(),
                    scan_blocks_buffer.as_entire_binding(),
                ))
            );

            let scan_blocks_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &scan_blocks_layout,
                &BindGroupEntries::sequential((
                    scan_blocks_buffer.as_entire_binding(),
                    scan_blocks_out_buffer.as_entire_binding(),
                ))
            );

            let compact_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &compact_layout,
                &BindGroupEntries::sequential((
                    instance_buffer.as_entire_binding(),
                    vote_buffer.as_entire_binding(),
                    scan_buffer.as_entire_binding(),
                    scan_blocks_out_buffer.as_entire_binding(),
                    compact_buffer.as_entire_binding(),
                    indirect_indexed_args_buffer.as_entire_binding(),
                ))
            );

 
            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                indirect_buffer: indirect_indexed_args_buffer,

                workgroup_count: chunk.workgroup_count,
                scan_workgroup_count: chunk.scan_workgroup_count,
                scan_blocks_workgroup_count: chunk.scan_groups_workgroup_count,

                compact_buffer,
                scan_bind_group,
                scan_blocks_bind_group,
                compact_bind_group,
            });
        }

        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }
}