use bevy::{prelude::*, render::{render_resource::{Buffer, BufferInitDescriptor, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferBinding}, renderer::RenderDevice}};

use super::{extract::{GrassInstanceData, GrassColorData}, pipeline::GrassPipeline};

#[derive(Component)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

pub(super) fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GrassInstanceData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Component)]
pub struct ColorBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_color_buffers(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &GrassColorData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, color) in &query {
        let layout = pipeline.color_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("color buffer"),
            contents: bytemuck::cast_slice(&[color.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("grass color bind group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                })
            }],
        });

        commands.entity(entity).insert(ColorBindGroup {
            bind_group,
        });
    }
}
