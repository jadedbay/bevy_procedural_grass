use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext}};

use crate::pipeline::GrassComputePipeline;

pub struct ComputeGrassNode;

impl render_graph::Node for ComputeGrassNode {
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        
        let pipeline_id = world.resource::<GrassComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline_id.compute_id)
            .unwrap();

        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_pipeline(pipeline);
            //TODO

        }

        Ok(())
    }
}