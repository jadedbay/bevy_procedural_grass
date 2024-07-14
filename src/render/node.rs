use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext}};

use crate::prelude::Grass;

use super::{pipeline::GrassComputePipeline, prepare::GrassComputeBindGroup};

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ComputeGrassNodeLabel;

pub(crate) struct ComputeGrassNode {
    query: QueryState<(&'static Grass, &'static GrassComputeBindGroup)>,
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

        for (grass, grass_compute_bind_group) in self.query.iter_manual(world) {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) {
                {
                    let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &grass_compute_bind_group.mesh_positions_bind_group, &[]);
                    pass.set_bind_group(1, &grass_compute_bind_group.chunk_indices_bind_groups[0], &[]);
                    pass.set_bind_group(2, &grass_compute_bind_group.grass_output_bind_group, &[]);
                    pass.dispatch_workgroups(grass.chunk_count.x, grass.chunk_count.y, 1);
                }
            }
        }

        Ok(())
    }
}