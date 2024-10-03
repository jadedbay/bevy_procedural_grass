use bevy::{core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey}, pbr::{MeshPipelineKey, PreparedMaterial, RenderMaterialInstances, RenderMeshInstances}, prelude::*, render::{mesh::GpuMesh, render_asset::RenderAssets, render_phase::{BinnedRenderPhaseType, DrawFunctions, ViewBinnedRenderPhases}, render_resource::{PipelineCache, SpecializedMeshPipelines}, view::ExtractedView}};

use crate::grass::{chunk::GrassChunk, material::GrassMaterial};

use super::{draw::DrawGrass, pipeline::GrassRenderPipeline};

pub(crate) fn queue_grass(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    grass_pipeline: Res<GrassRenderPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<GrassRenderPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<GpuMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    render_material_instances: Res<RenderMaterialInstances<GrassMaterial>>,
    render_materials: Res<RenderAssets<PreparedMaterial<GrassMaterial>>>,
    material_meshes: Query<Entity, With<GrassChunk>>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    mut views: Query<(Entity, &ExtractedView)>,
) {
    let draw_grass = opaque_3d_draw_functions.read().id::<DrawGrass>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view_entity, view) in &mut views {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        
        for entity in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let Some(material_asset_id) = render_material_instances.get(&entity) else {
                continue;
            };
            let Some(material) = render_materials.get(*material_asset_id) else {
                continue;
            };
            let key = view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines.specialize(&pipeline_cache, &grass_pipeline, key, &mesh.layout).unwrap();
            opaque_phase.add(
                Opaque3dBinKey {
                    pipeline,
                    draw_function: draw_grass,
                    asset_id: mesh_instance.mesh_asset_id.into(),
                    material_bind_group_id: material.get_bind_group_id().0,
                    lightmap_image: None,
                },
                entity,
                BinnedRenderPhaseType::UnbatchableMesh
            );
        }
    }
}