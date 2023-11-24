use bevy::{prelude::*, render::{render_phase::{DrawFunctions, RenderPhase}, render_resource::{SpecializedMeshPipelines, PipelineCache}, render_asset::RenderAssets, view::ExtractedView}, core_pipeline::core_3d::Opaque3d, pbr::{MeshUniform, MeshPipelineKey, RenderMeshInstances}};

use super::{pipeline::GrassPipeline, extract::GrassInstanceData, draw::DrawGrass, chunk::{GrassChunks, GrassToDraw}};

pub(super) fn grass_queue(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    custom_pipeline: Res<GrassPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<GrassPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<Entity, With<GrassToDraw>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_custom = opaque_3d_draw_functions.read().id::<DrawGrass>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
    for (view, mut opaque_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for entity in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.get(&entity) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            opaque_phase.add(Opaque3d {
                entity,
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder.distance_translation(&mesh_instance.transforms.transform.translation),
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
}