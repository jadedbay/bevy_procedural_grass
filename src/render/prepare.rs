use bevy::{prelude::*, render::{primitives::Aabb, render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::{chunk::{BoundingBox, GrassChunks}, ground_mesh::GroundMesh, Grass};
use super::{instance::GrassInstanceData, pipeline::{GrassComputePipeline, GrassComputeSPSPipelines}};

pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub instance_buffer: Buffer,
    pub blade_count: usize, // change this later

    pub workgroup_count: usize,

    pub compact_buffer: Buffer,
    pub sps_bind_group: BindGroup,
}

#[derive(Component)]
pub struct GrassBufferBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunks: Vec<GrassChunkBufferBindGroup>,
}

pub(crate) fn prepare_grass_bind_groups(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    sps_pipelines: Res<GrassComputeSPSPipelines>,
    query: Query<(Entity, &GrassChunks, &GroundMesh)>,
    render_device: Res<RenderDevice>
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let chunk_layout = pipeline.chunk_layout.clone();

    let sps_layout = sps_pipelines.sps_layout.clone();

    for (entity, chunks, ground_mesh) in query.iter() {

        let mesh_positions_bind_group = render_device.create_bind_group(
            Some("mesh_position_bind_group"),
            &mesh_layout,
            &BindGroupEntries::sequential((
                BufferBinding {
                    buffer: &ground_mesh.positions_buffer,
                    offset: 0,
                    size: None,
                },
                BufferBinding {
                    buffer: &ground_mesh.indices_buffer,
                    offset: 0,
                    size: None,
                }
            ))
        );

        let mut chunk_bind_groups = Vec::new();

        for (_, chunk) in chunks.0.clone() {
            let workgroup_count = chunk.indices_index.len();

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
                    size: (std::mem::size_of::<u32>() * workgroup_count * 8) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let instance_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("grass_data_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * workgroup_count * 8) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE, 
                    mapped_at_creation: false,
                }
            );

            let chunk_bind_group = render_device.create_bind_group(
                Some("indices_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    BufferBinding {
                        buffer: &aabb_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &indices_index_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &vote_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &instance_buffer,
                        offset: 0,
                        size: None,
                    }
                ))
            );

            let scan_output_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("scan_output_buffer"),
                    size: (std::mem::size_of::<u32>() * workgroup_count * 8) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }  
            );

            let compact_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("compact_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * workgroup_count * 8) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let sps_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &sps_layout,
                &BindGroupEntries::sequential((
                    BufferBinding {
                        buffer: &instance_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &vote_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &scan_output_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &compact_buffer,
                        offset: 0,
                        size: None,
                    }
                ))
            );
 
            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                instance_buffer,
                blade_count: workgroup_count * 8,
                workgroup_count,

                compact_buffer,
                sps_bind_group,
            });
        }


        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }
}