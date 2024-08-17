use bevy::{prelude::*, render::{render_graph::{self, NodeRunError, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext}};

use crate::prefix_sum::{prefix_sum_pass_vec, PrefixSumPipeline};

use super::{pipeline::GrassPrecomputePipeline, prepare::GroundMeshBindGroup};

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ComputeTriangleDispatchCountsLabel;

pub struct ComputeTriangleDispatchCountsNode {
    query: QueryState<&'static GroundMeshBindGroup>,    
}

impl FromWorld for ComputeTriangleDispatchCountsNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ComputeTriangleDispatchCountsNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world); 
    }

    fn run<'w>(
            &self,
            _graph: &mut RenderGraphContext,
            render_context: &mut RenderContext<'w>,
            world: &'w World,
        ) -> Result<(), NodeRunError> {
            let Some((scan_pipeline, scan_blocks_pipeline)) = PrefixSumPipeline::get_pipelines(world) else {
                return Ok(());
            };

            let pipeline_id = world.resource::<GrassPrecomputePipeline>(); 
            let pipeline_cache = world.resource::<PipelineCache>();
            
            let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.triangle_count_pipeline_id) else {
            return Ok(());
        };
            
            for bind_group in self.query.iter_manual(world) {
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &bind_group.bind_group, &[]);

                    pass.dispatch_workgroups((bind_group.triangle_count as f32 / 32.).ceil() as u32, 1, 1);
                }

                prefix_sum_pass_vec(render_context, &vec![bind_group.prefix_sum_bind_group.clone()], scan_pipeline, scan_blocks_pipeline);


                {

                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &bind_group.bind_group, &[]);

                    pass.dispatch_workgroups((bind_group.triangle_count as f32 / 32.).ceil() as u32, 1, 1);
                }
            } // TODO: make vec thingy nicer            

            Ok(())
        }
}
