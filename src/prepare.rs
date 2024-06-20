use std::marker::PhantomData;

use bevy::{prelude::*, render::{primitives::Aabb, render_resource::{BindGroup, BindGroupEntries, BufferBinding, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::pipeline::GrassPipeline;

#[derive(Component, Default)]
pub struct BindGroups<T> {
    bind_groups: Vec<BindGroup>,
    _phantom_data: PhantomData<T>,
}

pub(crate) fn prepare_compute_bind_groups(
    mut commands: Commands,
    pipeline: Res<GrassPipeline>,
    query: Query<(Entity, &Aabb)>,
    render_device: Res<RenderDevice>
) {
    let layout = pipeline.layout.clone();

    for (entity, aabb) in query.iter() {
        let aabb_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("aabb_buffer"),
            contents: bytemuck::cast_slice(&[Vec3::from(aabb.center), Vec3::from(aabb.half_extents)]), // TODO: use own struct for aabb
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        let aabb_bind_group =render_device.create_bind_group(
            Some("aabb_bind_group"),
            &layout,
            &BindGroupEntries::single(
                BufferBinding {
                    buffer: &aabb_buffer,
                    offset: 0,
                    size: None,
                }
            )
        );

        commands.entity(entity).insert(BindGroups::<Aabb> {
            bind_groups: vec![aabb_bind_group],
            ..default()
        });
    }
}