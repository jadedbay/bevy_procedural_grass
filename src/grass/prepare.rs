use std::time::Instant;

use bevy::{prelude::*, render::{render_resource::{Buffer, BufferInitDescriptor, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferBinding, BindGroupEntries}, renderer::RenderDevice, texture::FallbackImage, render_asset::RenderAssets}};

use super::{extract::{GrassInstanceData, GrassColorData, WindData, BladeData}, pipeline::GrassPipeline, wind::{WindMap, self}};

#[derive(Component)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
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
        let bind_group = render_device.create_bind_group(
            Some("grass color bind group"),
            &layout,
            &BindGroupEntries::single(BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            }),
        );

        commands.entity(entity).insert(ColorBindGroup {
            bind_group,
        });
    }
}

#[derive(Component)]
pub struct BladeBindGroup {
    pub bind_group: BindGroup,
}

pub(super) fn prepare_blade_buffers(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &BladeData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, blade) in &query {
        let layout = pipeline.blade_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("blade buffer"),
            contents: bytemuck::cast_slice(&[blade.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        let bind_group = render_device.create_bind_group(
            Some("blade bind group"),
            &layout,
            &BindGroupEntries::single(BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            })
        );

        commands.entity(entity).insert(BladeBindGroup {
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
) {
    for (entity, wind) in &query {
        let layout = pipeline.wind_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("wind buffer"),
            contents: bytemuck::cast_slice(&[wind.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        let bind_group = render_device.create_bind_group(
            Some("wind bind group"), 
            &layout,
            &BindGroupEntries::single(BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            })
        );

        commands.entity(entity).insert(WindBindGroup {
            bind_group,
        });
    }
}

#[derive(Component)]
pub struct WindMapBindGroup {
    pub bind_group: BindGroup,
}

pub fn prepare_wind_map_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<GrassPipeline>,
    fallback_img: Res<FallbackImage>,
    images: Res<RenderAssets<Image>>,
    query: Query<(Entity, &WindMap)>
) {
    let layout = pipeline.wind_map_layout.clone();

    for (entity, wind_map) in query.iter() {
        let wind_map_texture = if let Some(texture) = images.get(&wind_map.wind_map) {
            &texture.texture_view
        } else {
            &fallback_img.d2.texture_view
        };

        let bind_group = render_device.create_bind_group(
            Some("wind map bind group"),
            &layout,
            &BindGroupEntries::single(BindingResource::TextureView(&wind_map_texture))
        );
        commands.entity(entity).insert(WindMapBindGroup{
            bind_group,
        });
    }
}