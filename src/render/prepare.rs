use std::marker::PhantomData;

use bevy::{prelude::*, render::{mesh::GpuMesh, primitives::Aabb, render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntries, BufferBinding, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}};

use crate::grass::Grass;
use super::pipeline::GrassComputePipeline;

#[derive(Component, Default)]
pub struct BindGroups<T> {
    bind_groups: Vec<BindGroup>,
    _phantom_data: PhantomData<T>,
}

// pub(crate) fn prepare_compute_bind_groups(
//     mut commands: Commands,
//     meshes: ResMut<RenderAssets<GpuMesh>>,
//     pipeline: Res<GrassComputePipeline>,
//     query: Query<(Entity, &Grass, &Handle<Mesh>)>,
//     render_device: Res<RenderDevice>
// ) {
//     let layout = pipeline.mesh_layout.clone();

//     for (entity, grass, mesh_handle) in query.iter() {
//         let mesh = meshes.get(mesh_handle).unwrap()

//         let mesh_pos_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
//             label: Some("mesh_pos_buffer"),
//             contents: bytemuck::cast_slice(&[]),
//             usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
//         });

//         let aabb_bind_group = render_device.create_bind_group(
//             Some("aabb_bind_group"),
//             &layout,
//             &BindGroupEntries::single(
//                 BufferBinding {
//                     buffer: &aabb_buffer,
//                     offset: 0,
//                     size: None,
//                 }
//             )
//         );

//         commands.entity(entity).insert(BindGroups::<Aabb> {
//             bind_groups: vec![aabb_bind_group],
//             ..default()
//         });
//     }
// }