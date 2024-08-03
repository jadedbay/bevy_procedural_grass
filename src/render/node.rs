use bevy::{prelude::*, render::{camera::ExtractedCamera, render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::RenderContext, view::{ViewUniformOffset, ViewUniforms}}};

use super::{pipeline::{GrassComputeChunkPipeline, GrassComputeChunkPipelineId, GrassComputePPSPipelines, GrassComputePipeline}, prepare::GrassBufferBindGroup};

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ComputeGrassNodeLabel;


#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub enum NodeGrass {
    Counts,
    Compute, // run compute as a precompute step?
    Cull,
    PPS,
}

pub struct ComputeChunksNode {
    query: QueryState<(&'static GrassBufferBindGroup, &'static GrassComputeChunkPipelineId)>,
}
impl FromWorld for ComputeChunksNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}
impl render_graph::Node for ComputeChunksNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
         
        for (grass_bind_groups, pipeline_id) in self.query.iter_manual(world) {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.0) {
                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_pipeline(pipeline);
                pass.set_push_constants(0, &grass_bind_groups.triangle_count.to_le_bytes());
                pass.set_bind_group(0, &grass_bind_groups.mesh_bind_group, &[]);
                
                for chunk in &grass_bind_groups.chunksp {
                    pass.set_bind_group(1, &chunk.chunk_bind_group, &[]);
                    pass.dispatch_workgroups(grass_bind_groups.triangle_count, 0, 0); //TODO: 
                } 
            } 
        } 

        Ok(())
    }
}

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
        let sps_pipeline = world.resource::<GrassComputePPSPipelines>();
        let pipeline_cache = world.resource::<PipelineCache>();

        for grass_bind_groups in self.query.iter_manual(world) {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) {
                {
                    let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &grass_bind_groups.mesh_bind_group, &[]);

                    for chunk in &grass_bind_groups.chunks {
                        pass.set_bind_group(1, &chunk.chunk_bind_group, &[view_offset.offset]);
                        pass.dispatch_workgroups(chunk.workgroup_count as u32, 1, 1);
                    }
                }
            }
            
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(sps_pipeline.scan_pipeline) {
                {
                    let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());
                
                pass.set_pipeline(pipeline);
                
                    for chunk in &grass_bind_groups.chunks {
                        pass.set_bind_group(0, &chunk.scan_bind_group, &[]);
                        pass.dispatch_workgroups(chunk.scan_workgroup_count as u32, 1, 1)
                    }
                }
            }
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(sps_pipeline.scan_blocks_pipeline) {
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);

                    for chunk in &grass_bind_groups.chunks {
                        pass.set_push_constants(0, &(chunk.scan_workgroup_count as u32).to_le_bytes());
                        pass.set_bind_group(0, &chunk.scan_blocks_bind_group, &[]);
                        pass.dispatch_workgroups(chunk.scan_blocks_workgroup_count as u32, 1, 1)
                    }
                }
            }
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(sps_pipeline.compact_pipeline) {
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());

                    pass.set_pipeline(pipeline);

                    for chunk in &grass_bind_groups.chunks {
                        pass.set_bind_group(0, &chunk.compact_bind_group, &[]);
                        pass.dispatch_workgroups(chunk.scan_workgroup_count as u32, 1, 1);
                        
                    }
                }
            }
        }

        Ok(())
    }
}
