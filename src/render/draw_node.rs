use bevy::{pbr::RenderMeshInstances, prelude::*, render::{camera::ExtractedCamera, mesh::{GpuBufferInfo, GpuMesh}, render_asset::RenderAssets, render_graph::{NodeRunError, RenderLabel, ViewNode}, render_resource::{PipelineCache, RenderPassDescriptor, StoreOp}, view::{ViewDepthTexture, ViewTarget}}};

use super::{gpu_scene::GrassGpuScene, pipeline::GrassRenderPipeline};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct GrassDrawNodeLabel;


pub struct GrassDrawNode;
impl ViewNode for GrassDrawNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static ViewDepthTexture,
    );

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (camera, target, depth): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let Some(grass_gpu_scene) = world.get_resource::<GrassGpuScene>() else {
            return Ok(());
        };
        let depth_stencil_attachment = Some(depth.get_attachment(StoreOp::Store));
        
        let pipeline_id = world.resource::<GrassRenderPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("grass_pass"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: depth_stencil_attachment.clone(),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        // render_pass.set_render_pipeline(pipeline_id.mesh_pipeline.into());
        
        for (entity, grass_bind_group) in &grass_gpu_scene.entities {
            let Some(mesh_instance) = world.resource::<RenderMeshInstances>().render_mesh_queue_data(*entity) else {
                return Ok(())
            };
            
            let Some(gpu_mesh) = world.resource::<RenderAssets<GpuMesh>>().get(mesh_instance.mesh_asset_id) else {
                return Ok(())
            };
            
            if let Some(viewport) = camera.viewport.as_ref() {
                render_pass.set_camera_viewport(viewport);
            };
            render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count: _,
                } => {
                    render_pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    for chunk in &grass_bind_group.chunks {
                        render_pass.set_vertex_buffer(1, chunk.compact_buffer.slice(..));
                        render_pass.draw_indexed_indirect(&chunk.indirect_buffer, 0);
                    }
                }
                GpuBufferInfo::NonIndexed => {} // will always be indexed
            }
        }


        Ok(())
    }
}

impl FromWorld for GrassDrawNode {
    fn from_world(_world: &mut World) -> Self {
        Self
    }
}