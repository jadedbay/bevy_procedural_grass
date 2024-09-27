use bevy::{prelude::*, render::{camera::ExtractedCamera, render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{CachedPipelineState, CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderDevice, RenderQueue}, view::{ViewUniformOffset, ViewUniforms}}};

use crate::prefix_sum::{prefix_sum_pass_vec, PrefixSumPipeline};

use super::{pipeline::GrassComputePipeline, prepare::{GrassBufferBindGroup, GrassEntities, GrassStage}};

enum ComputeGrassState {
    Loading,
    Loaded
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ComputeGrassNodeLabel;

pub struct ComputeGrassNode {
    state: ComputeGrassState,
    query: QueryState<&'static GrassBufferBindGroup>,
}

impl FromWorld for ComputeGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: ComputeGrassState::Loading,
            query: QueryState::new(world),
        }
    }
}


// impl render_graph::Node for ComputeGrassNode {
//     fn update(&mut self, world: &mut World) {
//         self.query.update_archetypes(world);

//         match self.state {
//             ComputeGrassState::Loading => {
//                 let pipeline_cache = world.resource::<PipelineCache>();
//                 let compute_pipeline = world.resource::<GrassComputePipeline>();

//                 match pipeline_cache.get_compute_pipeline_state(compute_pipeline.cull_pipeline_id) {
//                     CachedPipelineState::Ok(_) => {
//                         self.state = ComputeGrassState::Loaded;
//                     }
//                     CachedPipelineState::Err(err) => {
//                         panic!("Error initializing cull grass pipeline: {err}");
//                     }
//                     _ => {}
//                 }
//             }
//             ComputeGrassState::Loaded => {}
//         }
//     }
    
//     fn run<'w>(
//         &self,
//         _graph: &mut RenderGraphContext,
//         render_context: &mut RenderContext<'w>,
//         world: &'w World,
//     ) -> Result<(), render_graph::NodeRunError> {
//         match self.state {
//             ComputeGrassState::Loading => {}
//             ComputeGrassState::Loaded => {
//                 let pipeline_id = world.resource::<GrassComputePipeline>();
//                 let pipeline_cache = world.resource::<PipelineCache>();
                
//                 let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else {
//                     return Ok(());
//                 };

//                 for grass_bind_groups in self.query.iter_manual(world) {
//                     {
//                         let mut pass = render_context
//                         .command_encoder()
//                         .begin_compute_pass(&ComputePassDescriptor::default());
        
//                         pass.set_pipeline(compute_pipeline);
//                         pass.set_bind_group(0, &grass_bind_groups.mesh_bind_group, &[]);
        
//                         for chunk in &grass_bind_groups.chunks {
//                             pass.set_bind_group(1, &chunk.chunk_bind_group, &[]);
//                             pass.dispatch_workgroups(chunk.workgroup_count as u32, 1, 1);
//                         }
//                     }      
//                 }
//             }
//         }

//         Ok(())
//     }
// }

pub fn compute_grass(
    query: Query<(Entity, &GrassBufferBindGroup)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_id: Res<GrassComputePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut grass_entities: ResMut<GrassEntities>,
) {
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else { return; };

    for (entity, grass_bind_groups) in query.iter() {
        if !grass_entities.0.contains_key(&entity) {
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_pipeline(compute_pipeline);
            for chunk in &grass_bind_groups.chunks {
                pass.set_bind_group(0, &chunk.chunk_bind_group.as_ref().unwrap(), &[]);
                pass.dispatch_workgroups(chunk.workgroup_count as u32, 1, 1);
            }

            grass_entities.0.insert(entity, GrassStage::Cull);
        }
    }

    render_queue.submit([command_encoder.finish()]);
}

enum CullGrassState {
    Loading,
    Loaded
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct CullGrassNodeLabel;

pub struct CullGrassNode {
    state: CullGrassState,
    query: QueryState<&'static GrassBufferBindGroup>,
    view_offset_query: QueryState<&'static ViewUniformOffset>,
}

impl FromWorld for CullGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: CullGrassState::Loading,
            query: QueryState::new(world),
            view_offset_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for CullGrassNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.view_offset_query.update_archetypes(world);

        match self.state {
            CullGrassState::Loading => {
                let pipeline_cache = world.resource::<PipelineCache>();
                let compute_pipeline = world.resource::<GrassComputePipeline>();
                let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
                
                let pipeline_states = [
                    pipeline_cache.get_compute_pipeline_state(compute_pipeline.compute_id),
                    pipeline_cache.get_compute_pipeline_state(compute_pipeline.compact_pipeline_id),
                    pipeline_cache.get_compute_pipeline_state(compute_pipeline.cull_pipeline_id),
                    pipeline_cache.get_compute_pipeline_state(prefix_sum_pipeline.scan_pipeline),
                    pipeline_cache.get_compute_pipeline_state(prefix_sum_pipeline.scan_blocks_pipeline),
                ];

                if pipeline_states.iter().all(|state| matches!(state, CachedPipelineState::Ok(_))) {
                    self.state = CullGrassState::Loaded;
                } else if pipeline_states.iter().any(|state| {
                    if let CachedPipelineState::Err(err) = state {
                        panic!("Error initializing one or more grass pipelines: {}", err);
                    }
                    false
                }) {
                    unreachable!();
                }
            }
            CullGrassState::Loaded => {}
        }
    }

    fn run<'w>(
            &self,
            graph: &mut RenderGraphContext,
            render_context: &mut RenderContext<'w>,
            world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        match self.state {
            CullGrassState::Loading => {}
            CullGrassState::Loaded => {
                let Ok(view_offset) = self.view_offset_query.get_manual(world, graph.view_entity()) else { return Ok(()); };
                
                let pipeline_id = world.resource::<GrassComputePipeline>();
                let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
                let pipeline_cache = world.resource::<PipelineCache>();
                
                let Some(cull_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.cull_pipeline_id) else {
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

                        pass.set_pipeline(cull_pipeline);
                        for chunk in &grass_bind_groups.chunks {
                            pass.set_bind_group(0, &chunk.cull_bind_group, &[view_offset.offset]);
                            pass.dispatch_workgroups(chunk.cull_workgroup_count, 1, 1);
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
            }
        }

        Ok(())
    }
}