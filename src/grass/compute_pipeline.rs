use std::borrow::Cow;

use bevy::{prelude::*, render::{render_resource::{BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, PipelineCache, ComputePipelineDescriptor, BufferDescriptor, BufferUsages, BindGroupEntries, BufferBinding, Buffer, BindGroup, CachedPipelineState, ComputePassDescriptor, BufferInitDescriptor}, renderer::{RenderDevice, RenderContext}, render_graph, render_asset::RenderAssets, mesh::VertexAttributeValues}, ecs::system::lifetimeless::SRes};

#[derive(Resource)]
pub struct GrassComputePipeline {
    grass_data_bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for GrassComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let grass_data_bind_group_layout = 
            render_device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE | ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/grass_compute.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![grass_data_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        GrassComputePipeline {
            grass_data_bind_group_layout,
            pipeline,
        }
    }
}

#[derive(Resource)]
pub struct GrassDataBindGroup{
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

pub fn prepare_compute_bind_group(
    mut commands: Commands,
    pipeline: Res<GrassComputePipeline>,
    render_device: Res<RenderDevice>,
) {
    let buffer = render_device.create_buffer(&BufferDescriptor {
        label: None,
        size: 32 * 32 * 32,
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE | BufferUsages::VERTEX,
        mapped_at_creation: true,
    });

    let bind_group = render_device.create_bind_group(
        None, 
        &pipeline.grass_data_bind_group_layout, 
        &BindGroupEntries::single(BufferBinding {
            buffer: &buffer,
            offset: 0,
            size: None,
        }),
    );


    commands.insert_resource(
        GrassDataBindGroup {
            buffer,
            bind_group,
        }
    );
}

pub enum ComputeState {
    Loading,
    Update,
}

pub struct ComputeNode {
    state: ComputeState,
}

impl Default for ComputeNode {
    fn default() -> Self {
        Self {
            state: ComputeState::Loading,
        }
    }
}

impl render_graph::Node for ComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GrassComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            ComputeState::Loading => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    self.state = ComputeState::Update;
                }
            }
            ComputeState::Update => {}
        }
    }
    
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let grass_data = &world.resource::<GrassDataBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GrassComputePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &grass_data.bind_group, &[]);

        match self.state {
            ComputeState::Loading => {}
            ComputeState::Update => {
                let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline.pipeline).unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(1, 1, 1);
                world.resource::<GrassDataBindGroup>().buffer.unmap();
            }
        }

        Ok(())
    }
}