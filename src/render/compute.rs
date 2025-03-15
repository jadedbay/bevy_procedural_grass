use bevy::{pbr::{PreparedMaterial, RenderMaterialInstances}, prelude::*, render::{render_asset::RenderAssets, render_resource::{CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache}, renderer::{RenderDevice, RenderQueue}, sync_world::MainEntity}};

use crate::{grass::clump::GrassClumpsBindGroup, prelude::GrassMaterial};

use super::{pipeline::GrassGeneratePipelineId, prepare::{ComputedGrassEntities, GrassChunkComputeBindGroup}};

pub fn compute_grass(
    query: Query<(&MainEntity, &GrassChunkComputeBindGroup, &GrassGeneratePipelineId)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut grass_entities: ResMut<ComputedGrassEntities>,
    clumps: Option<Res<GrassClumpsBindGroup>>,
    materials: Res<RenderAssets<PreparedMaterial<GrassMaterial>>>,
    material_instances: Res<RenderMaterialInstances<GrassMaterial>>,
) {
    if query.is_empty() { return; } 
    pipeline_cache.process_queue();

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());

    // TODO: OPTIMIZATION move pass creation before loop and set appropriate bind groups
    for (main_entity, bind_group, pipeline_id) in query.iter() {
        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.0) else { return; };
        let Some(material_asset_id) = material_instances.get(main_entity) else { continue; };
        let Some(material) = materials.get(*material_asset_id) else { continue; };

        let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(compute_pipeline);

        pass.set_bind_group(0, &bind_group.bind_group, &[]);
        pass.set_bind_group(1, &material.bind_group, &[]);
        if let Some(ref clumps) = clumps {
            pass.set_bind_group(2, &clumps.bind_group, &[]);
        }
        pass.dispatch_workgroups(bind_group.workgroup_count, 1, 1);

        // grass_entities.0.push(entity);
    }
    render_queue.submit([command_encoder.finish()]);
}
