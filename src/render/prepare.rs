use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::Grass;
use super::{compute_mesh::GrassGroundMesh, instance::GrassInstanceData, pipeline::GrassComputePipeline};

pub struct GrassChunkBufferBindGroup {
    pub indices_bind_group: BindGroup,
    pub grass_data_bind_group: BindGroup,
    pub grass_data_buffer: Buffer,
    pub grass_data_length: usize,

    pub triangle_count: usize,
}

#[derive(Component)]
pub struct GrassBufferBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunks: Vec<GrassChunkBufferBindGroup>,
}

pub(crate) fn prepare_grass_bind_groups(
    mut commands: Commands,
    grass_base_meshes: ResMut<RenderAssets<GrassGroundMesh>>,
    pipeline: Res<GrassComputePipeline>,
    ground_query: Query<&Handle<Mesh>>,
    query: Query<(Entity, &Grass)>,
    render_device: Res<RenderDevice>
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let indices_layout = pipeline.indices_layout.clone();
    let grass_data_layout = pipeline.grass_output_layout.clone();

    for (entity, grass) in query.iter() {
        let mesh_positions_bind_group = render_device.create_bind_group(
            Some("mesh_position_bind_group"),
            &mesh_layout,
            &BindGroupEntries::single(
                BufferBinding {
                    buffer: &grass_base_meshes.get(ground_query.get(grass.ground_entity.unwrap()).unwrap()).unwrap().positions_buffer,
                    offset: 0,
                    size: None,
                }
            )
        );

        let mut chunk_bind_groups = Vec::new();

        for (_, chunk) in grass.chunks.clone() {
            let triangle_count = chunk.mesh_indices.len() / 3;

            let indices_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_buffer"),
                    contents: bytemuck::cast_slice(chunk.mesh_indices.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            let indices_bind_group = render_device.create_bind_group(
                Some("indices_bind_group"),
                &indices_layout,
                &BindGroupEntries::single(
                    BufferBinding {
                        buffer: &indices_buffer,
                        offset: 0,
                        size: None,
                    }
                )
            );
 
            let grass_data_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("grass_data_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * triangle_count * 32) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE, 
                    mapped_at_creation: false,
                }
            );

            let grass_data_bind_group = render_device.create_bind_group(
                Some("output_bind_group"),
                &grass_data_layout,
                &BindGroupEntries::single(
                    BufferBinding {
                        buffer: &grass_data_buffer,
                        offset: 0,
                        size: None,
                    }
                )
            );

            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                indices_bind_group,
                grass_data_bind_group,
                grass_data_buffer,
                grass_data_length: triangle_count * 32,
                triangle_count,
            });
        }


        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }
}