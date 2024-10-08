use bevy::{ecs::query::QueryIter, prelude::*, render::{render_resource::{binding_types::{storage_buffer, storage_buffer_read_only}, BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, Buffer, BufferDescriptor, BufferUsages, CachedComputePipelineId, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages}, renderer::{RenderContext, RenderDevice}}};

use crate::render::prepare::GrassChunkCullBindGroups;

#[derive(Resource)]
pub struct PrefixSumPipeline {
    pub scan_layout: BindGroupLayout,
    pub scan_blocks_layout: BindGroupLayout,
    pub scan_pipeline: CachedComputePipelineId,
    pub scan_blocks_pipeline: CachedComputePipelineId,
}

impl FromWorld for PrefixSumPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let scan_layout = render_device.create_bind_group_layout(
            "scan_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                )
            )
        );

        let scan_blocks_layout = render_device.create_bind_group_layout(
            "scan_blocks_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer_read_only::<Vec<u32>>(false),
                    storage_buffer::<Vec<u32>>(false),
                )
            )
        );


        let scan_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/scan.wgsl");
        let scan_blocks_shader = world.resource::<AssetServer>().load("embedded://bevy_procedural_grass/shaders/scan_blocks.wgsl");

        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let scan_pipeline = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("compute_scan_grass_pipeline".into()),
                layout: vec![scan_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: scan_shader.clone(),
                shader_defs: vec![],
                entry_point: "scan".into(),
        });

        let scan_blocks_pipeline = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("compute_scan_blocks_pipeline".into()),
                layout: vec![scan_blocks_layout.clone()],
                push_constant_ranges: vec![PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4,
                }],
                shader: scan_blocks_shader.clone(),
                shader_defs: vec![],
                entry_point: "scan_blocks".into(),
            });
         
        Self {
            scan_layout,
            scan_blocks_layout,
            scan_pipeline,
            scan_blocks_pipeline,

        }
    }
}

impl PrefixSumPipeline {
    pub fn get_pipelines(
        world: &World,
    ) -> Option<(
        &ComputePipeline,
        &ComputePipeline,
    )> {
        let pipeline_cache = world.get_resource::<PipelineCache>()?;
        let pipeline = world.get_resource::<Self>()?;

        Some((
            pipeline_cache.get_compute_pipeline(pipeline.scan_pipeline)?,
            pipeline_cache.get_compute_pipeline(pipeline.scan_blocks_pipeline)?,
        ))
    }
}



pub fn prefix_sum_pass(
    render_context: &mut RenderContext,
    chunks: Vec<(&GrassChunkCullBindGroups, &PrefixSumBindGroups)>,
    // query_iter: QueryIter<'_, '_, (&GrassChunkCullBindGroups, &PrefixSumBindGroups), ()>,
    scan_pipeline: &ComputePipeline,
    scan_blocks_pipeline: &ComputePipeline,
) {

    {
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(scan_pipeline);
        for (_, bind_groups) in &chunks {
            pass.set_bind_group(0, &bind_groups.scan_bind_group, &[]);
            pass.dispatch_workgroups(bind_groups.scan_workgroups, 1, 1);
        }
    }
    {
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(scan_blocks_pipeline);
        for (_, bind_groups) in &chunks {
        pass.set_push_constants(0, &(bind_groups.scan_workgroups as u32).to_le_bytes());
        pass.set_bind_group(0, &bind_groups.scan_blocks_bind_group, &[]);
        pass.dispatch_workgroups(bind_groups.scan_blocks_workgroups, 1, 1);
        }
    }
}

#[derive(Clone)]
pub(crate) struct PrefixSumBuffers {
    pub scan_buffer: Buffer,
    pub scan_blocks_buffer: Buffer,
    pub scan_blocks_out_buffer: Buffer,
}

impl PrefixSumBuffers {
    pub fn create_buffers(
        render_device: &RenderDevice,
        input_length: u32,
        scan_workgroups: u32,
    ) -> Self {
        let scan_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("scan_buffer"),
            size: (std::mem::size_of::<u32>() * input_length as usize) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let scan_blocks_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("scan_blocks_buffer"),
            size: (std::mem::size_of::<u32>() * scan_workgroups as usize) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
    
        let scan_blocks_out_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("scan_blocks_out_buffer"),
            size: (std::mem::size_of::<u32>() * scan_workgroups as usize) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
    
        Self {
            scan_buffer,
            scan_blocks_buffer,
            scan_blocks_out_buffer,
        }
    }
}

#[derive(Component, Clone)]
pub struct PrefixSumBindGroups {
    pub scan_bind_group: BindGroup,
    pub scan_blocks_bind_group: BindGroup,

    pub scan_workgroups: u32,
    pub scan_blocks_workgroups: u32,
}

impl PrefixSumBindGroups {
    pub fn create_bind_groups(
        render_device: &RenderDevice,
        pipeline: &PrefixSumPipeline,
        input_buffer: &Buffer,
        buffers: &PrefixSumBuffers,
        scan_workgroups: u32,
        scan_blocks_workgroups: u32,
    ) -> Self {  
        let scan_bind_group = render_device.create_bind_group( 
            Some("scan_bind_group"),
            &pipeline.scan_layout,
            &BindGroupEntries::sequential((
                input_buffer.as_entire_binding(),
                buffers.scan_buffer.as_entire_binding(),
                buffers.scan_blocks_buffer.as_entire_binding(),
            )),
        );
    
        let scan_blocks_bind_group = render_device.create_bind_group(
            Some("scan_blocks_bind_group"),
            &pipeline.scan_blocks_layout,
            &BindGroupEntries::sequential((
                buffers.scan_blocks_buffer.as_entire_binding(),
                buffers.scan_blocks_out_buffer.as_entire_binding(),
            )),
        );
     
        Self {
            scan_bind_group,
            scan_blocks_bind_group,
            
            scan_workgroups,
            scan_blocks_workgroups,
        }
    }
}

pub fn calculate_workgroup_counts(count: u32) -> (u32, u32) {
    let mut scan_workgroup_count = (count as f32 / 128.).ceil() as u32;
    if scan_workgroup_count > 128 {
        let mut p2 = 128;
        while p2 < scan_workgroup_count {
            p2 *= 2;
        }

        scan_workgroup_count = p2;
    } else {
        while 128 % scan_workgroup_count != 0 {
            scan_workgroup_count += 1;
        }
    }

    let scan_groups_workgroup_count = (count as f32 / 1024.).ceil() as u32;

    (scan_workgroup_count, scan_groups_workgroup_count)
}
