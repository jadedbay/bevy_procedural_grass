use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{CachedPipelineState, CommandEncoderDescriptor, ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderDevice, RenderQueue}, view::ViewUniformOffset}};

use crate::{prefix_sum::{prefix_sum_pass, PrefixSumBindGroups, PrefixSumPipeline}, prelude::GrassConfig};

use super::{pipeline::GrassComputePipeline, prepare::{ComputedGrassEntities, GrassChunkComputeBindGroup, GrassChunkCullBindGroups, GrassShadowBindGroups, ShadowPrefixSumBindGroups}};

pub fn compute_grass(
    query: Query<(Entity, &GrassChunkComputeBindGroup)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_id: Res<GrassComputePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut grass_entities: ResMut<ComputedGrassEntities>,
) {
    if query.is_empty() { return; } // encoder/submit is expensive

    let mut command_encoder = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.compute_id) else { return; };

    for (entity, bind_group) in query.iter() {
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
            
            pass.set_pipeline(compute_pipeline);
            
            pass.set_bind_group(0, &bind_group.bind_group, &[]);
            pass.dispatch_workgroups(bind_group.workgroup_count, 1, 1);
            
            grass_entities.0.push(entity);
    }
    render_queue.submit([command_encoder.finish()]);
}

enum NodeState {
    Loading,
    Loaded
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct CullGrassNodeLabel;

pub struct CullGrassNode {
    state: NodeState,
    query: QueryState<(&'static GrassChunkCullBindGroups, &'static PrefixSumBindGroups)>, 
    shadow_query: QueryState<(&'static GrassShadowBindGroups, &'static ShadowPrefixSumBindGroups)>,
    view_offset_query: QueryState<&'static ViewUniformOffset>,
}

impl FromWorld for CullGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: NodeState::Loading,
            query: QueryState::new(world),
            shadow_query: QueryState::new(world),
            view_offset_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for CullGrassNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.shadow_query.update_archetypes(world);
        self.view_offset_query.update_archetypes(world);

        match self.state {
            NodeState::Loading => {
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
                    self.state = NodeState::Loaded;
                } else if pipeline_states.iter().any(|state| {
                    if let CachedPipelineState::Err(err) = state {
                        panic!("Error initializing one or more grass pipelines: {}", err);
                    }
                    false
                }) {
                    unreachable!();
                }
            }
            NodeState::Loaded => {}
        }
    }

    fn run<'w>(
            &self,
            graph: &mut RenderGraphContext,
            render_context: &mut RenderContext<'w>,
            world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        match self.state {
            NodeState::Loading => {}
            NodeState::Loaded => {
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

                prefix_sum_pass(render_context, self.query.iter_manual(world).collect(), prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);
                
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                
                    pass.set_pipeline(compact_pipeline);
                    
                    for (grass_bind_groups, _) in self.query.iter_manual(world) {
                        pass.set_bind_group(0, &grass_bind_groups.compact_bind_group, &[]);
                        pass.dispatch_workgroups(grass_bind_groups.compact_workgroup_count as u32, 1, 1); 
                    }
                }

                let shadow_bind_groups: Vec<_> = self.shadow_query.iter_manual(world)
                    .map(|(shadow_bind_groups, shadow_prefix_sum_bind_groups)| 
                        (&shadow_bind_groups.0, &shadow_prefix_sum_bind_groups.0)
                    )
                    .collect();
                prefix_sum_pass(render_context, shadow_bind_groups, prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);

                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());
            
                pass.set_pipeline(compact_pipeline);
                
                for (grass_bind_groups, _) in self.shadow_query.iter_manual(world) {
                    pass.set_bind_group(0, &grass_bind_groups.0.compact_bind_group, &[]);
                    pass.dispatch_workgroups(grass_bind_groups.0.compact_workgroup_count as u32, 1, 1); 
                }
            }
        }
        
        Ok(())
    }
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct ResetArgsNodeLabel;

pub struct ResetArgsNode {
    state: NodeState,
    query: QueryState<&'static GrassChunkCullBindGroups>,
    shadow_query: QueryState<&'static GrassShadowBindGroups>,
}

impl FromWorld for ResetArgsNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: NodeState::Loading,
            query: QueryState::new(world),
            shadow_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ResetArgsNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.shadow_query.update_archetypes(world);
        
        match self.state {
            NodeState::Loading => {
                let pipeline_cache = world.resource::<PipelineCache>();
                let compute_pipeline = world.resource::<GrassComputePipeline>();

                match pipeline_cache.get_compute_pipeline_state(compute_pipeline.reset_args_pipeline_id) {
                    CachedPipelineState::Ok(_) => {
                        self.state = NodeState::Loaded;
                    }
                    CachedPipelineState::Err(err) => {
                        panic!("Failed initialising reset args pipeline {err}");
                    }
                    _ => {}
                } 
            }
            NodeState::Loaded => {}
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        match self.state {
            NodeState::Loading => {}
            NodeState::Loaded => {
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
                for grass_bind_groups in self.shadow_query.iter_manual(world) {
                    pass.set_bind_group(0, &grass_bind_groups.0.reset_args_bind_group, &[]);
                    pass.dispatch_workgroups(1, 1, 1);
                }
            }
        }
        Ok(())
    }
}