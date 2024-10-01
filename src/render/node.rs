use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{CachedPipelineState, CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderDevice, RenderQueue}, view::ViewUniformOffset}};

use crate::prefix_sum::{prefix_sum_pass, PrefixSumBindGroups, PrefixSumPipeline};

use super::{pipeline::GrassComputePipeline, prepare::{ComputeGrassMarker, GrassChunkBufferBindGroup, GrassEntities, GrassStage}};

pub fn compute_grass(
    query: Query<(Entity, &GrassChunkBufferBindGroup), With<ComputeGrassMarker>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_id: Res<GrassComputePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut grass_entities: ResMut<GrassEntities>,
) {
    if query.is_empty() { return; }
    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else { return; };

    for (entity, bind_groups) in query.iter() {
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
            
            pass.set_pipeline(compute_pipeline);
            
            pass.set_bind_group(0, &bind_groups.chunk_bind_group.as_ref().unwrap(), &[]);
            pass.dispatch_workgroups(bind_groups.workgroup_count, 1, 1);
            
            grass_entities.0.insert(entity, GrassStage::Cull);
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
    query: QueryState<(&'static GrassChunkBufferBindGroup, &'static PrefixSumBindGroups)>,
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
                    pipeline_cache.get_compute_pipeline_state(compute_pipeline.reset_args_pipeline_id),
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

                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                
                    pass.set_pipeline(cull_pipeline);
                    for (grass_bind_groups, _) in self.query.iter_manual(world) {
                        pass.set_bind_group(0, &grass_bind_groups.cull_bind_group, &[view_offset.offset]);
                        pass.dispatch_workgroups(grass_bind_groups.cull_workgroup_count, 1, 1);
                    }
                }

                prefix_sum_pass(render_context, self.query.iter_manual(world), prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);

                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());
            
                pass.set_pipeline(compact_pipeline);
                
                for (grass_bind_groups, _) in self.query.iter_manual(world) {
                    pass.set_bind_group(0, &grass_bind_groups.compact_bind_group, &[]);
                    pass.dispatch_workgroups(grass_bind_groups.compact_workgroup_count as u32, 1, 1); 
                }
            }
        }
        
        Ok(())
    }
}

enum ResetArgsNodeState {
    Loading,
    Loaded,
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ResetArgsNodeLabel;

pub struct ResetArgsNode {
    state: ResetArgsNodeState,
    query: QueryState<&'static GrassChunkBufferBindGroup>,
}

impl FromWorld for ResetArgsNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: ResetArgsNodeState::Loading,
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ResetArgsNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        
        match self.state {
            ResetArgsNodeState::Loading => {
                let pipeline_cache = world.resource::<PipelineCache>();
                let compute_pipeline = world.resource::<GrassComputePipeline>();

                match pipeline_cache.get_compute_pipeline_state(compute_pipeline.reset_args_pipeline_id) {
                    CachedPipelineState::Ok(_) => {
                        self.state = ResetArgsNodeState::Loaded;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Failed initialising reset args pipeline {err}");
                    }
                    _ => {}
                } 
            }
            ResetArgsNodeState::Loaded => {}
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        match self.state {
            ResetArgsNodeState::Loading => {}
            ResetArgsNodeState::Loaded => {
                let pipeline_id = world.resource::<GrassComputePipeline>();
                let pipeline_cache = world.resource::<PipelineCache>();

                let Some(reset_args_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.reset_args_pipeline_id) else {
                    return Ok(());
                };

                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());
 
                pass.set_pipeline(reset_args_pipeline);
                for grass_bind_groups in self.query.iter_manual(world) {
                    pass.set_bind_group(0, &grass_bind_groups.reset_args_bind_group, &[]);
                    pass.dispatch_workgroups(1, 1, 1);
                }
            }
        }
        Ok(())
    }
}