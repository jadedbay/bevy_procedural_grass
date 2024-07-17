use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::Grass;
use super::{compute_mesh::GrassGroundMesh, instance::GrassInstanceData, pipeline::GrassComputePipeline};

pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub output_buffer: Buffer,
    pub blade_count: usize, // change this later

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
    let chunk_layout = pipeline.chunk_layout.clone();

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

            let output_buffer = render_device.create_buffer(
                &BufferDescriptor {
                    label: Some("grass_data_buffer"),
                    size: (std::mem::size_of::<GrassInstanceData>() * triangle_count * 64) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE, 
                    mapped_at_creation: false,
                }
            );

            let chunk_bind_group = render_device.create_bind_group(
                Some("indices_bind_group"),
                &chunk_layout,
                &BindGroupEntries::sequential((
                    BufferBinding {
                        buffer: &indices_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &output_buffer,
                        offset: 0,
                        size: None,
                    }
                ))
            );
 
            chunk_bind_groups.push(GrassChunkBufferBindGroup {
                chunk_bind_group,
                output_buffer,
                blade_count: triangle_count * 64,
                triangle_count,
            });
        }


        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }
}