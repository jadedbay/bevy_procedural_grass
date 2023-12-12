use bevy::{prelude::*, render::{render_phase::{SetItemPipeline, PhaseItem, RenderCommand, TrackedRenderPass, RenderCommandResult}, render_asset::RenderAssets, mesh::GpuBufferInfo}, pbr::{SetMeshViewBindGroup, SetMeshBindGroup, RenderMeshInstances}, ecs::system::{lifetimeless::{SRes, Read}, SystemParamItem}};

use crate::grass::{wind::GrassWind, chunk::{RenderGrassChunks, GrassLOD}, grass::{Grass, GrassLODMesh}};

use super::{prepare::BufferBindGroup, instance::GrassInstanceData};

pub type DrawGrass = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetGrassBindGroup<2>,
    SetWindBindGroup<3>,
    DrawGrassInstanced,
);

pub struct SetGrassBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetGrassBindGroup<I> 
{
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<BufferBindGroup<Grass>>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w BufferBindGroup<Grass>>,
        _meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(bind_group) = bind_group else {
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(I, &bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetWindBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetWindBindGroup<I> 
{
    type Param = SRes<BufferBindGroup<GrassWind>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<BufferBindGroup<GrassWind>>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        local_wind: Option<&'w BufferBindGroup<GrassWind>>,
        wind_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_group = if let Some(local_wind) = local_wind {
            local_wind
        } else {
            wind_bind_group.into_inner()
        };
        pass.set_bind_group(I, &bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawGrassInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawGrassInstanced {
    type Param = (SRes<RenderAssets<Mesh>>, SRes<RenderMeshInstances>, SRes<RenderAssets<GrassInstanceData>>);
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<GrassLODMesh>, Read<RenderGrassChunks>);

    #[inline]
    fn render<'w>( 
        item: &P,
        _view: (),
        (lod, chunks): (&'w GrassLODMesh, &'w RenderGrassChunks),
        (meshes, render_mesh_instances, grass_data): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(mesh_instance) = render_mesh_instances.get(&item.entity()) else {
            return RenderCommandResult::Failure;
        };

        let meshes = meshes.into_inner();

        let gpu_mesh_high = match meshes.get(mesh_instance.mesh_asset_id) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };
        
        let gpu_mesh_low = if let Some(lod) = &lod.mesh_handle {
            match meshes.get(lod) {
                Some(gpu_mesh) => gpu_mesh,
                None => return RenderCommandResult::Failure,
            }
        } else {
            gpu_mesh_high
        };

        let grass_data_inner = grass_data.into_inner();

        for chunk in &chunks.0 {
            let gpu_grass = match grass_data_inner.get(chunk.1.clone()) {
                Some(gpu_grass) => gpu_grass,
                None => return RenderCommandResult::Failure,
            };

            let gpu_mesh = match chunk.0 {
                GrassLOD::Low => &gpu_mesh_low,
                GrassLOD::High => &gpu_mesh_high,
            };

            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, gpu_grass.buffer.slice(..));

            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..gpu_grass.length as u32);
                }
                GpuBufferInfo::NonIndexed => {
                    pass.draw(0..gpu_mesh.vertex_count, 0..gpu_grass.length as u32);
                }
            }
        }
        
        RenderCommandResult::Success
    }
}