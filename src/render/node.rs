use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel}, render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache}, renderer::RenderContext, view::ViewUniformOffset}};

use crate::prefix_sum::{prefix_sum_pass, PrefixSumBindGroups, PrefixSumPipeline};

use super::{pipeline::{GrassCompactPipeline, GrassCullPipelineId}, prepare::{CompactBindGroups, CompactBindGroupsLOD, GrassChunkCullBindGroup, GrassShadowBindGroups, PrefixSumBindGroupsLOD, ShadowPrefixSumBindGroups}};

enum NodeState {
    Loading,
    Loaded
}

#[derive(RenderLabel, Hash, Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct CullGrassNodeLabel;

pub struct CullGrassNode {
    state: NodeState,
    query: QueryState<(&'static GrassChunkCullBindGroup, &'static CompactBindGroups, &'static PrefixSumBindGroups, &'static GrassCullPipelineId)>,
    lod_query: QueryState<(&'static CompactBindGroupsLOD, &'static PrefixSumBindGroupsLOD)>,
    shadow_query: QueryState<(&'static GrassShadowBindGroups, &'static ShadowPrefixSumBindGroups)>,
    view_offset_query: QueryState<&'static ViewUniformOffset>,
}

impl FromWorld for CullGrassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: NodeState::Loading,
            query: QueryState::new(world),
            lod_query: QueryState::new(world),
            shadow_query: QueryState::new(world),
            view_offset_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for CullGrassNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.lod_query.update_archetypes(world);
        self.shadow_query.update_archetypes(world);
        self.view_offset_query.update_archetypes(world);

        match self.state {
            NodeState::Loading => {
                let pipeline_cache = world.resource::<PipelineCache>();
                let compute_pipeline = world.resource::<GrassCompactPipeline>();
                let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
                
                let pipeline_states = [
                    pipeline_cache.get_compute_pipeline_state(compute_pipeline.compact_pipeline_id),
                    // pipeline_cache.get_compute_pipeline_state(compute_pipeline.cull_pipeline_id),
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
                
                let pipeline_id = world.resource::<GrassCompactPipeline>();
                let prefix_sum_pipeline = world.resource::<PrefixSumPipeline>();
                let pipeline_cache = world.resource::<PipelineCache>();
 
                // let Some(shadow_cull_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.shadows_cull_pipeline_id) else {
                //     return Ok(());
                // };

                // let Some(cull_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.cull_pipeline_id) else {
                //     return Ok(());
                // };
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
                    
                    for (grass_bind_groups, _, _, pipeline_id) in self.query.iter_manual(world) {
                        let Some(cull_pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id.0) else {
                            return Ok(());
                        };

                        pass.set_pipeline(cull_pipeline);
                        pass.set_bind_group(0, &grass_bind_groups.cull_bind_group, &[view_offset.offset]);
                        pass.dispatch_workgroups(grass_bind_groups.cull_workgroup_count, 1, 1);
                    }
                }
                let bind_groups: Vec<_> = self.query.iter_manual(world)
                    .map(|(_, bind_groups, prefix_sum_bind_groups, _)| 
                        (bind_groups, prefix_sum_bind_groups)
                    )
                    .collect();
                // HIGH
                prefix_sum_pass(render_context, bind_groups, prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                
                    pass.set_pipeline(compact_pipeline);
                    
                    for (_, grass_bind_groups, _, _) in self.query.iter_manual(world) {
                        pass.set_bind_group(0, &grass_bind_groups.compact_bind_group, &[]);
                        pass.dispatch_workgroups(grass_bind_groups.compact_workgroup_count as u32, 1, 1); 
                    }
                }
                // LOW
                let lod_bind_groups: Vec<_> = self.lod_query.iter_manual(world)
                    .map(|(lod_bind_groups, lod_prefix_sum_bind_groups)| 
                        (&lod_bind_groups.0, &lod_prefix_sum_bind_groups.0)
                    )
                    .collect();
                prefix_sum_pass(render_context, lod_bind_groups, prefix_sum_scan_pipeline, prefix_sum_scan_blocks_pipeline);
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                
                    pass.set_pipeline(compact_pipeline);
                    
                    for (grass_bind_groups, _) in self.lod_query.iter_manual(world) {
                        pass.set_bind_group(0, &grass_bind_groups.0.compact_bind_group, &[]);
                        pass.dispatch_workgroups(grass_bind_groups.0.compact_workgroup_count as u32, 1, 1); 
                    }
                }
                // SHADOW
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
    query: QueryState<&'static CompactBindGroups>,
    lod_query: QueryState<&'static CompactBindGroupsLOD>,
    shadow_query: QueryState<&'static GrassShadowBindGroups>,
}

impl FromWorld for ResetArgsNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            state: NodeState::Loading,
            query: QueryState::new(world),
            lod_query: QueryState::new(world),
            shadow_query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for ResetArgsNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.lod_query.update_archetypes(world);
        self.shadow_query.update_archetypes(world);
        
        match self.state {
            NodeState::Loading => {
                let pipeline_cache = world.resource::<PipelineCache>();
                let compute_pipeline = world.resource::<GrassCompactPipeline>();

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
                let pipeline_id = world.resource::<GrassCompactPipeline>();
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
                for grass_bind_groups in self.lod_query.iter_manual(world) {
                    pass.set_bind_group(0, &grass_bind_groups.0.reset_args_bind_group, &[]);
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