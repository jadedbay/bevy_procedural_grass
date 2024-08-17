use bevy::{prelude::*, render::{render_resource::{BindGroup, BindGroupEntries, BufferDescriptor, BufferInitDescriptor, BufferUsages}, renderer::RenderDevice}, utils::HashMap};

use crate::{grass::ground_mesh::GroundMesh, prefix_sum::{create_prefix_sum_bind_group_buffers, PrefixSumBindGroup, PrefixSumPipeline}};

use super::pipeline::GrassPrecomputePipeline;

#[derive(Default)]
pub enum GrassPreComputeStep {
    #[default]
    PreReadback,
    PostReadback,
}

#[derive(Resource)]
pub struct GrassPrecomputeEntities(HashMap<Entity, GrassPreComputeStep>);

#[derive(Component, Clone)]
pub struct GroundMeshBindGroup {
    pub bind_group: BindGroup,
    pub prefix_sum_bind_group: PrefixSumBindGroup,
    pub expand_bind_group: BindGroup,
    pub triangle_count: usize,
}

pub fn prepare_ground_mesh_bindgroup(
    mut commands: Commands,
    pipeline: Res<GrassPrecomputePipeline>,
    prefix_sum_pipeline: Res<PrefixSumPipeline>,
    query: Query<(Entity, &GroundMesh)>,
    render_device: Res<RenderDevice>,
) {    
    for (entity, ground_mesh) in &query {
        let dispatch_counts_buffer = render_device.create_buffer(
            &BufferDescriptor {
                label: Some("dispatch_counts_buffer"),
                size: (std::mem::size_of::<u32>() * ground_mesh.triangle_count) as u64,
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            }
        );

        let density_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("density_buffer"),
                contents: bytemuck::cast_slice(&[0.5]),
                usage: BufferUsages::UNIFORM,
            }
        );

        let mesh_bind_group = render_device.create_bind_group(
            Some("mesh_bind_group"),
            &pipeline.mesh_layout,
            &BindGroupEntries::sequential((
                ground_mesh.positions_buffer.as_entire_binding(),
                ground_mesh.indices_buffer.as_entire_binding(),
                dispatch_counts_buffer.as_entire_binding(),
                density_buffer.as_entire_binding(),
            ))
        );

        // let prefix_sum_bind_group = create_prefix_sum_bind_group_buffers(
        //     &render_device, 
        //     &prefix_sum_pipeline, 
        //     &dispatch_counts_buffer, 
        //     ground_mesh.triangle_count as u32,
        // );


        // commands.entity(entity).insert(GroundMeshBindGroup {
        //     bind_group: mesh_bind_group,
        //     prefix_sum_bind_group,
        //     triangle_count: ground_mesh.triangle_count,
        // });
    }
}

