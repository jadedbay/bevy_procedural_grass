use bevy::{prelude::*, render::{render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages, DrawIndexedIndirectArgs}, renderer::RenderDevice}};

use crate::grass::{chunk::{BoundingBox, GrassChunks}, ground_mesh::GroundMesh};
use super::{gpu_scene::{GrassGpuScene, GrassGpuSceneMarker}, instance::GrassInstanceData, pipeline::{GrassComputePPSPipelines, GrassComputePipeline}};

#[derive(Clone)]
pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub indirect_buffer: Buffer,

    pub workgroup_count: usize,
    pub scan_workgroup_count: usize,
    pub scan_blocks_workgroup_count: usize,


    pub instance_buffer: Buffer,
    pub vote_buffer: Buffer,
    pub scan_buffer: Buffer,
    pub scan_blocks_buffer: Buffer,
    pub compact_buffer: Buffer,
    pub scan_bind_group: BindGroup,
    pub scan_blocks_bind_group: BindGroup,
    pub compact_bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct GrassBufferBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunks: Vec<GrassChunkBufferBindGroup>,
}

pub(crate) fn prepare_grass_bind_groups(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    sps_pipelines: Res<GrassComputePPSPipelines>,
    query: Query<(Entity, &GrassChunks, &GroundMesh)>,
    render_device: Res<RenderDevice>,
    mut grass_gpu_scene: ResMut<GrassGpuScene>,
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let chunk_layout = pipeline.chunk_layout.clone();

    let scan_layout = sps_pipelines.scan_layout.clone();
    let scan_blocks_layout = sps_pipelines.scan_blocks_layout.clone();
    let compact_layout = sps_pipelines.compact_layout.clone();

    for (entity, chunks, ground_mesh) in query.iter() {
        dbg!(grass_gpu_scene.entities.keys());
        if grass_gpu_scene.entities.contains_key(&entity) {
            for chunk in &mut grass_gpu_scene.entities.get_mut(&entity).unwrap().chunks {
 
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

                let compact_bind_group = render_device.create_bind_group(
                    Some("scan_bind_group"),
                    &compact_layout,
                    &BindGroupEntries::sequential((
                        BufferBinding { buffer: &chunk.instance_buffer, offset: 0, size: None },
                        BufferBinding { buffer: &chunk.vote_buffer, offset: 0, size: None },
                        BufferBinding { buffer: &chunk.scan_buffer, offset: 0, size: None },
                        BufferBinding { buffer: &chunk.scan_blocks_buffer, offset: 0, size: None },
                        BufferBinding { buffer: &chunk.compact_buffer, offset: 0, size: None }, 
                        BufferBinding { buffer: &indirect_indexed_args_buffer, offset: 0, size: None },
                    ))
                );

                chunk.compact_bind_group = compact_bind_group;
            }
            commands.entity(entity).insert(grass_gpu_scene.entities.get(&entity).unwrap().clone());
            continue;
        }

        dbg!("yo");
                
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
                    },
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
                    BufferBinding { buffer: &vote_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &scan_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &scan_blocks_buffer, offset: 0, size: None },
                ))
            );

            let scan_blocks_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &scan_blocks_layout,
                &BindGroupEntries::sequential((
                    BufferBinding { buffer: &scan_blocks_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &scan_blocks_out_buffer, offset: 0, size: None },
                ))
            );

            let compact_bind_group = render_device.create_bind_group(
                Some("scan_bind_group"),
                &compact_layout,
                &BindGroupEntries::sequential((
                    BufferBinding { buffer: &instance_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &vote_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &scan_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &scan_blocks_out_buffer, offset: 0, size: None },
                    BufferBinding { buffer: &compact_buffer, offset: 0, size: None }, 
                    BufferBinding { buffer: &indirect_indexed_args_buffer, offset: 0, size: None },
                ))
            );

 
            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                indirect_buffer: indirect_indexed_args_buffer,

                workgroup_count: chunk.workgroup_count,
                scan_workgroup_count: chunk.scan_workgroup_count,
                scan_blocks_workgroup_count: chunk.scan_groups_workgroup_count,

                instance_buffer,
                scan_buffer,
                vote_buffer,
                scan_blocks_buffer: scan_blocks_out_buffer,
                compact_buffer,
                scan_bind_group,
                scan_blocks_bind_group,
                compact_bind_group,
            });
        }

        dbg!("hi");

        grass_gpu_scene.entities.insert(entity, GrassBufferBindGroup {
            mesh_positions_bind_group: mesh_positions_bind_group.clone(),
            chunks: chunk_bind_groups.clone(),
        });

        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }

    commands.spawn(GrassGpuSceneMarker);
}