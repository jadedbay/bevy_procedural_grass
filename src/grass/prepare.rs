use std::time::Instant;

use bevy::{prelude::*, render::{render_resource::{Buffer, BufferInitDescriptor, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferBinding}, renderer::RenderDevice}};

use super::{extract::{GrassInstanceData, GrassColorData, WindData, LightData}, pipeline::GrassPipeline};

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

#[derive(Component)]
pub struct WindBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_wind_buffers(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &WindData)>,
    render_device: Res<RenderDevice>,
    time: Res<Time>,
) {
    for (entity, wind) in &query {
        let layout = pipeline.wind_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("wind buffer"),
            contents: bytemuck::cast_slice(&[wind.frequency, wind.speed, wind.noise, wind.strength, time.elapsed_seconds()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("wind bind group"),
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

        commands.entity(entity).insert(WindBindGroup {
            bind_group,
        });
    }
}

#[derive(Component)]
pub struct LightBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_light_buffers(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &LightData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, light_data) in &query {
        let layout = pipeline.light_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("light buffer"),
            contents: bytemuck::cast_slice(&[light_data.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("light bind group"),
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

        commands.entity(entity).insert(LightBindGroup {
            bind_group,
        });
    }
}