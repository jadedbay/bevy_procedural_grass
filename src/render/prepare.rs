use std::marker::PhantomData;

use bevy::{prelude::*, render::{render_resource::{BufferInitDescriptor, BufferUsages, BindGroup, BindingResource, BufferBinding, BindGroupEntries, Buffer}, renderer::RenderDevice, texture::FallbackImage, render_asset::RenderAssets}};

use crate::grass::{wind::GrassWind, grass::{Blade, GrassColor, Grass}};

use super::pipeline::GrassPipeline;

#[derive(Component, Resource, Clone)]
pub struct BufferBindGroup<T> {
    pub bind_group: BindGroup,
    _marker: PhantomData<T>,
}

impl<T> BufferBindGroup<T> {
    pub fn new(bind_group: BindGroup) -> Self {
        Self {
            bind_group,
            _marker:  PhantomData,
        }
    }
}

#[derive(Component, Clone)]
pub struct GrassBuffer {
    pub color_buffer: Buffer,
    pub blade_buffer: Buffer,
}

pub(crate) fn prepare_grass_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GrassColor, &Blade)>,
    render_device: Res<RenderDevice>
) {
    for (entity, color, blade) in &query {
        let color_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("color buffer"),
            contents: bytemuck::cast_slice(&color.to_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        let blade_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("blade buffer"),
            contents: bytemuck::cast_slice(&[blade.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        commands.entity(entity).insert(GrassBuffer {
            color_buffer,
            blade_buffer,
        });
    }
}

pub(crate) fn prepare_grass_bind_group(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &GrassBuffer)>, 
) {
    let layout = pipeline.grass_layout.clone();

    for (entity, grass) in query.iter() {
        let bind_group = render_device.create_bind_group(
            Some("grass bind group"),
            &layout,
            &BindGroupEntries::sequential((
                BufferBinding {
                    buffer: &grass.color_buffer,
                    offset: 0,
                    size: None,
                },
                BufferBinding {
                    buffer: &grass.blade_buffer,
                    offset: 0,
                    size: None,
                }
            )),
        );

        commands.entity(entity).insert(BufferBindGroup::<Grass>::new(bind_group));
    }
}

#[derive(Component, Resource, Clone)]
pub struct WindBuffer {
    buffer: Buffer,
}

pub(crate) fn prepare_global_wind_buffers(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    wind: Res<GrassWind>,
) {
    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("wind buffer"),
        contents: bytemuck::cast_slice(&[wind.wind_data.clone()]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
    });

    commands.insert_resource(WindBuffer {
        buffer,
    });
}

pub(crate) fn prepare_global_wind_bind_group(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    render_device: Res<RenderDevice>,
    wind: Res<GrassWind>,
    wind_buffer: Res<WindBuffer>,
    fallback_img: Res<FallbackImage>,
    images: Res<RenderAssets<Image>>,
) {
    let layout = pipeline.wind_layout.clone();

    let wind_map_texture = if let Some(texture) = images.get(&wind.wind_map) {
        &texture.texture_view
    } else {
        &fallback_img.d2.texture_view
    };

    let bind_group = render_device.create_bind_group(
        Some("wind bind group"), 
        &layout,
        &BindGroupEntries::sequential((
            BufferBinding {
                buffer: &wind_buffer.buffer,
                offset: 0,
                size: None,
            },
            BindingResource::TextureView(&wind_map_texture)
        ))
    );

    commands.insert_resource(BufferBindGroup::<GrassWind>::new(bind_group));
}

pub(crate) fn prepare_local_wind_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GrassWind)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, grass_wind) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("local wind buffer"),
            contents: bytemuck::cast_slice(&[grass_wind.wind_data.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        commands.entity(entity).insert(WindBuffer {
            buffer,
        });
    }
}

pub(crate) fn prepare_local_wind_bind_group(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &GrassWind, &WindBuffer)>,
    fallback_img: Res<FallbackImage>,
    images: Res<RenderAssets<Image>>,
) {
    let layout = pipeline.wind_layout.clone();

    for (entity, grass_wind, wind_buffer) in query.iter() {
        let wind_map_texture = if let Some(texture) = images.get(&grass_wind.wind_map) {
            &texture.texture_view
        } else {
            &fallback_img.d2.texture_view
        };

        let bind_group = render_device.create_bind_group(
            Some("local wind bind group"),
            &layout,
            &BindGroupEntries::sequential((
                BufferBinding {
                    buffer: &wind_buffer.buffer,
                    offset: 0,
                    size: None,
                },
                BindingResource::TextureView(&wind_map_texture)
            ))
        );

        commands.entity(entity).insert(BufferBindGroup::<GrassWind>::new(bind_group));
    }
}