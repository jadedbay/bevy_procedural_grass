use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext}};

use super::{pipeline::GrassComputePipeline, prepare::GrassBufferBindGroup};

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ComputeGrassNodeLabel;

pub(crate) struct ComputeGrassNode {
    query: QueryState<&'static GrassBufferBindGroup>,
}

impl FromWorld for ComputeGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ComputeGrassNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_id = world.resource::<GrassComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        for grass_bind_groups in self.query.iter_manual(world) {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) {
                {
                    let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &grass_bind_groups.mesh_positions_bind_group, &[]);

                    for chunk in &grass_bind_groups.chunks {
                        pass.set_bind_group(1, &chunk.indices_bind_group, &[]);
                        pass.set_bind_group(2, &chunk.grass_data_bind_group, &[]);
                        pass.dispatch_workgroups(1, 1, 1);
                    }
                }
            }
        }

        Ok(())
    }
}