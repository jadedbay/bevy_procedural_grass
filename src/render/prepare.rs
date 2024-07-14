use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::Grass;

use super::{compute_mesh::GrassGroundMesh, pipeline::GrassComputePipeline};

#[derive(Component)]
pub struct GrassBufferBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunk_indices_bind_groups: Vec<BindGroup>,
    pub grass_data_bind_group: BindGroup,
    pub grass_data_buffer: Buffer,
    pub length: usize,
}

pub(crate) fn prepare_compute_bind_groups(
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
                    buffer: &grass_base_meshes.get(ground_query.get(grass.ground_entity).unwrap()).unwrap().positions_buffer,
                    offset: 0,
                    size: None,
                }
            )
        );

        let mut chunk_indices_bind_groups = Vec::new();

        for (_, chunk) in grass.chunks.clone() {
            let buffer = render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("indices_buffer"),
                    contents: bytemuck::cast_slice(chunk.mesh_indices.as_slice()),
                    usage: BufferUsages::STORAGE,
                }
            );

            let bind_group = render_device.create_bind_group(
                Some("indices_bind_group"),
                &indices_layout,
                &BindGroupEntries::single(
                    BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }
                )
            );

            chunk_indices_bind_groups.push(bind_group);
        }

        let grass_data_buffer = render_device.create_buffer(
            &BufferDescriptor {
                label: Some("grass_data_buffer"),
                size: std::mem::size_of::<[f32; 8]>() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
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

        commands.entity(entity).insert(GrassBufferBindGroup {
            mesh_positions_bind_group,
            chunk_indices_bind_groups,
            grass_data_bind_group,
            grass_data_buffer,
            length: 2,
        });
    }
}