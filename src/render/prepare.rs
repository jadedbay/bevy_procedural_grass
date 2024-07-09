use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, Buffer, BufferBinding, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::Grass;

use super::{mesh_asset::GrassBaseMesh, pipeline::GrassComputePipeline};

#[derive(Component)]
pub struct GrassComputeBindGroup {
    pub mesh_positions_bind_group: BindGroup,
    pub chunk_indices_bind_groups: Vec<BindGroup>,
}

pub(crate) fn prepare_compute_bind_groups(
    mut commands: Commands,
    grass_base_meshes: ResMut<RenderAssets<GrassBaseMesh>>,
    pipeline: Res<GrassComputePipeline>,
    query: Query<(Entity, &Handle<Mesh>, &Grass)>,
    render_device: Res<RenderDevice>
) {
    let mesh_layout = pipeline.mesh_layout.clone();
    let indices_layout = pipeline.indices_layout.clone();

    for (entity, mesh_handle, grass) in query.iter() {
        let mesh_positions_bind_group = render_device.create_bind_group(
            Some("mesh_position_bind_group"),
            &mesh_layout,
            &BindGroupEntries::single(
                BufferBinding {
                    buffer: &grass_base_meshes.get(mesh_handle).unwrap().positions_buffer,
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
                        offset:0,
                        size: None,
                    }
                )
            );

            chunk_indices_bind_groups.push(bind_group);
        }

        commands.entity(entity).insert(GrassComputeBindGroup {
            mesh_positions_bind_group,
            chunk_indices_bind_groups,
        });
    }
}

#[derive(Component)]
pub(crate) struct GrassInstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

fn prepare_grass_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &Grass)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass) in &query {
        let data: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; //temp

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("grass_instance_buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: BufferUsages::STORAGE,
        });

        commands.entity(entity).insert(GrassInstanceBuffer {
            buffer,
            length: 1,
        });
    }
}