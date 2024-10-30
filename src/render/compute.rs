use bevy::{pbr::{PreparedMaterial, RenderMaterialInstances}, prelude::*, render::{render_asset::RenderAssets, render_resource::{CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache}, renderer::{RenderDevice, RenderQueue}}};

use crate::{grass::clump::GrassClumpsGpu, prelude::GrassMaterial};

use super::{pipeline::GrassComputePipeline, prepare::{ComputedGrassEntities, GrassChunkComputeBindGroup}};

pub fn compute_grass(
    query: Query<(Entity, &GrassChunkComputeBindGroup)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_id: Res<GrassComputePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut grass_entities: ResMut<ComputedGrassEntities>,
    clumps: Res<GrassClumpsGpu>,
    materials: Res<RenderAssets<PreparedMaterial<GrassMaterial>>>,
    material_instances: Res<RenderMaterialInstances<GrassMaterial>>,
) {
    if query.is_empty() { return; } 

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else { return; };

    // TODO: OPTIMIZATION move pass creation before loop and set appropriate bind groups
    for (entity, bind_group) in query.iter() {
            let Some(material_asset_id) = material_instances.get(&entity) else { continue; };
            let Some(material) = materials.get(*material_asset_id) else { continue; };

            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
            
            pass.set_pipeline(compute_pipeline);
            
            pass.set_bind_group(0, &bind_group.bind_group, &[]);
            pass.set_bind_group(1, &clumps.bind_group, &[]);
            pass.set_bind_group(2, &material.bind_group, &[]);
            pass.dispatch_workgroups(bind_group.workgroup_count, 1, 1);
            
            grass_entities.0.push(entity);
    }
    render_queue.submit([command_encoder.finish()]);
}