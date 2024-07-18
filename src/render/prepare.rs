use bevy::{prelude::*, render::{primitives::Aabb, render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::{chunk::GrassChunks, Grass};
use super::{compute_mesh::GrassGroundMesh, instance::GrassInstanceData, pipeline::GrassComputePipeline};

pub struct GrassChunkBufferBindGroup {
    pub chunk_bind_group: BindGroup,
    pub output_buffer: Buffer,
    pub blade_count: usize, // change this later

    pub workgroup_count: usize,
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
    query: Query<(Entity, &Grass, &GrassChunks)>,
    render_device: Res<RenderDevice>
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let chunk_layout = pipeline.chunk_layout.clone();

    for (entity, grass, chunks) in query.iter() {
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

        for (_, chunk) in chunks.0.clone() {
            let workgroup_count = chunk.indices_index.len();

            let aabb_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("aabb_buffer"),
                    contents: bytemuck::cast_slice(&[BoundingBox::from(chunk.aabb)]),
                    usage: BufferUsages::UNIFORM,
                }
            );

            let indices_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_buffer"),
                    contents: bytemuck::cast_slice(chunk.mesh_indices.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            let indices_index_buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_index_buffer"),
                    contents: bytemuck::cast_slice(chunk.indices_index.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            let output_buffer = render_device.create_buffer(
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
                        buffer: &indices_buffer,
                        offset: 0,
                        size: None,
                    },
                    BufferBinding {
                        buffer: &indices_index_buffer,
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
                blade_count: workgroup_count * 8,
                workgroup_count,
            });
        }


        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunks: chunk_bind_groups,
        });
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct BoundingBox {
    min: Vec3,
    _padding: f32,
    max: Vec3,
    _padding2: f32,
}

impl From<Aabb> for BoundingBox {
    fn from(aabb: Aabb) -> Self {
        Self {
            min: aabb.min().into(),
            _padding: 0.0,
            max: aabb.max().into(),
            _padding2: 0.0,
        }
    }
}