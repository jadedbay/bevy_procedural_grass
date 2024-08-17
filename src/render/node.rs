use bevy::{prelude::*, render::{camera::ExtractedCamera, render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext, view::{ViewUniformOffset, ViewUniforms}}};

use crate::prefix_sum::{prefix_sum_pass_vec, PrefixSumPipeline};

use super::{pipeline::GrassComputePipeline, prepare::GrassBufferBindGroup};

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ComputeGrassNodeLabel;

pub struct ComputeGrassNode {
    query: QueryState<&'static GrassBufferBindGroup>,
    view_offset_query: QueryState<&'static ViewUniformOffset>,
}

impl FromWorld for ComputeGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            view_offset_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ComputeGrassNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.view_offset_query.update_archetypes(world);
    }
    
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let Ok(view_offset) = self.view_offset_query.get_manual(world, graph.view_entity()) else { return Ok(()); };

        let pipeline_id = world.resource::<GrassComputePipeline>();
        let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        
        let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else {
            return Ok(());
        };
        let Some(prefix_sum_scan_pipeline) = pipeline_cache.get_compute_pipeline(prefix_sum_pipeline.scan_pipeline) else {
            return Ok(());
        };
        let Some(prefix_sum_scan_blocks_pipeline) = pipeline_cache.get_compute_pipeline(prefix_sum_pipeline.scan_blocks_pipeline) else {
            return Ok(());
        };
        let Some(compact_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compact_pipeline_id) else {
            return Ok(());
        };

        for grass_bind_groups in self.query.iter_manual(world) {
            {
                let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_pipeline(compute_pipeline);
                pass.set_bind_group(0, &grass_bind_groups.mesh_bind_group, &[]);

                for chunk in &grass_bind_groups.chunks {
                    pass.set_bind_group(1, &chunk.chunk_bind_group, &[view_offset.offset]);
                    pass.dispatch_workgroups(chunk.workgroup_count as u32, 1, 1);
                }
            }
            
            prefix_sum_pass_vec(render_context, &grass_bind_groups.prefix_sum_chunks, prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);
            
            {
                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_pipeline(compact_pipeline);

                for chunk in &grass_bind_groups.chunks {
                    pass.set_bind_group(0, &chunk.compact_bind_group, &[]);
                    pass.dispatch_workgroups(chunk.compact_workgroup_count as u32, 1, 1); 
                }
            }
            
        }

        Ok(())
    }
}
