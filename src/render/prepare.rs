use std::marker::PhantomData;

use bevy::{prelude::*, render::{render_resource::{BufferInitDescriptor, BufferUsages, BindGroup, BindingResource, BufferBinding, BindGroupEntries}, renderer::RenderDevice, texture::FallbackImage, render_asset::RenderAssets}};

use crate::grass::{wind::WindMap, grass::{Blade, GrassColor, Grass}};

use super::{extract::WindData, pipeline::GrassPipeline};

#[derive(Component)]
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

pub(crate) fn prepare_grass_buffers(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &GrassColor, &Blade)>,
    render_device: Res<RenderDevice>
) {
    for (entity, color, blade) in &query {
        let layout = pipeline.grass_layout.clone();

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

        let bind_group = render_device.create_bind_group(
            Some("grass bind group"),
            &layout,
            &BindGroupEntries::sequential((
                BufferBinding {
                    buffer: &color_buffer,
                    offset: 0,
                    size: None,
                },
                BufferBinding {
                    buffer: &blade_buffer,
                    offset: 0,
                    size: None,
                }
            )),
        );

        commands.entity(entity).insert(BufferBindGroup::<Grass>::new(bind_group));
    }
}

pub(crate) fn prepare_wind_buffers(
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

        commands.entity(entity).insert(BufferBindGroup::<WindData>::new(bind_group));
    }
}

pub(crate) fn prepare_wind_map_buffers(
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
        commands.entity(entity).insert(BufferBindGroup::<WindMap>::new(bind_group));
    }
}