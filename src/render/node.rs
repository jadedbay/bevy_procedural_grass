use bevy::{prelude::*, render::{render_graph::{self, RenderGraphContext, RenderLabel, SlotInfo, SlotType}, render_resource::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, Maintain, MapMode, PipelineCache}, renderer::{RenderContext, RenderDevice, RenderQueue}}};

use super::{pipeline::{GrassComputePipeline, GrassComputeSPSPipelines}, prepare::GrassBufferBindGroup};

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
        let sps_pipeline = world.resource::<GrassComputeSPSPipelines>();
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
                        pass.set_bind_group(1, &chunk.chunk_bind_group, &[]);
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
                        pass.set_bind_group(0, &chunk.sps_bind_group, &[]);
                        pass.dispatch_workgroups((chunk.workgroup_count / 8) as u32, 1, 1)
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
                        pass.set_bind_group(0, &chunk.sps_bind_group, &[]);
                        pass.dispatch_workgroups((chunk.workgroup_count / 8) as u32, 1, 1)
                    }
                }
            }
        }

        Ok(())
    }
}

fn read_instance_count(
    render_device: &RenderDevice,
    render_queue: &RenderQueue,
    scan_buffer: &Buffer,
    instance_count: usize,
) -> u32 {
    let staging_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("staging_buffer"),
        size: std::mem::size_of::<u32>() as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("copy_encode"),
    });

    encoder.copy_buffer_to_buffer(
        &scan_buffer,
        (instance_count * std::mem::size_of::<u32>()) as u64 - std::mem::size_of::<u32>() as u64,
        &staging_buffer,
        0,
        std::mem::size_of::<u32>() as u64,
    );

    render_queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (s, r) = crossbeam_channel::unbounded::<()>();

    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    render_device.poll(Maintain::wait()).panic_on_timeout();
    r.recv().expect("Failed to receive the map_async message");
    
    let data = buffer_slice.get_mapped_range();
    let instance_count = bytemuck::cast_slice::<u8, u32>(&data)[0];
    staging_buffer.unmap();

    instance_count
}