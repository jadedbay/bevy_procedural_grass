use bevy::{core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey}, pbr::{CascadesVisibleEntities, CubemapVisibleEntities, ExtractedDirectionalLight, ExtractedPointLight, LightEntity, MaterialPipeline, MaterialPipelineKey, MeshPipelineKey, PreparedMaterial, PrepassPipeline, RenderLightmaps, RenderMaterialInstances, RenderMeshInstanceFlags, RenderMeshInstances, Shadow, ShadowBinKey, ViewLightEntities}, prelude::*, render::{mesh::GpuMesh, render_asset::RenderAssets, render_phase::{BinnedRenderPhaseType, DrawFunctions, ViewBinnedRenderPhases}, render_resource::{PipelineCache, SpecializedMeshPipelines}, view::{ExtractedView, VisibleEntities, WithMesh}}};

use crate::{grass::{chunk::GrassChunk, config::GrassLightType, material::GrassMaterial}, prelude::GrassConfig};

use super::draw::{DrawGrass, DrawGrassPrepass};

pub(crate) fn queue_grass(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    grass_pipeline: Res<MaterialPipeline<GrassMaterial>>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<MaterialPipeline<GrassMaterial>>>,
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
            let key = view_key 
                | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology())
                | material.properties.mesh_pipeline_key_bits;

            let pipeline = pipelines.specialize(
                &pipeline_cache, 
                &grass_pipeline, 
                MaterialPipelineKey {
                    mesh_key: key,
                    bind_group_data: material.key.clone(),
                }, 
                &mesh.layout
            ).unwrap();
            
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

// Copied from bevy to add custom draw function
pub fn queue_grass_shadows(
    shadow_draw_functions: Res<DrawFunctions<Shadow>>,
    prepass_pipeline: Res<PrepassPipeline<GrassMaterial>>,
    render_meshes: Res<RenderAssets<GpuMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    render_materials: Res<RenderAssets<PreparedMaterial<GrassMaterial>>>,
    render_material_instances: Res<RenderMaterialInstances<GrassMaterial>>,
    mut shadow_render_phases: ResMut<ViewBinnedRenderPhases<Shadow>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<PrepassPipeline<GrassMaterial>>>,
    pipeline_cache: Res<PipelineCache>,
    view_lights: Query<(Entity, &ViewLightEntities)>,
    mut view_light_entities: Query<&LightEntity>,
    point_light_entities: Query<&CubemapVisibleEntities, With<ExtractedPointLight>>,
    directional_light_entities: Query<&CascadesVisibleEntities, With<ExtractedDirectionalLight>>,
    spot_light_entities: Query<&VisibleEntities, With<ExtractedPointLight>>,
    grass_config: Res<GrassConfig>,
) {
    for (entity, view_lights) in &view_lights {

        let draw_shadow_mesh = shadow_draw_functions.read().id::<DrawGrassPrepass>();

        for view_light_entity in view_lights.lights.iter().copied() {
            let Ok(light_entity) = view_light_entities.get_mut(view_light_entity) else {
                continue;
            };

            let light_type = match light_entity {
                LightEntity::Directional { .. } => GrassLightType::Directional,
                LightEntity::Point { .. } => GrassLightType::Point,
                LightEntity::Spot { .. } => GrassLightType::Spot,
            };

            if !grass_config.grass_shadows.light_enabled(light_type) {
                continue;
            }

            let Some(shadow_phase) = shadow_render_phases.get_mut(&view_light_entity) else {
                continue;
            };

            // TODO: add options in GrassConfig for shadows from types of lights (and fix shader)
            let is_directional_light = matches!(light_entity, LightEntity::Directional { .. });
            let visible_entities = match light_entity {
                LightEntity::Directional {
                    light_entity,
                    cascade_index,
                } => directional_light_entities
                    .get(*light_entity)
                    .expect("Failed to get directional light visible entities")
                    .entities
                    .get(&entity)
                    .expect("Failed to get directional light visible entities for view")
                    .get(*cascade_index)
                    .expect("Failed to get directional light visible entities for cascade"),
                LightEntity::Point {
                    light_entity,
                    face_index,
                } => point_light_entities
                    .get(*light_entity)
                    .expect("Failed to get point light visible entities")
                    .get(*face_index),
                LightEntity::Spot { light_entity } => spot_light_entities
                    .get(*light_entity)
                    .expect("Failed to get spot light visible entities"),
            };
            let mut light_key = MeshPipelineKey::DEPTH_PREPASS;
            light_key.set(MeshPipelineKey::DEPTH_CLAMP_ORTHO, is_directional_light);


            // NOTE: Lights with shadow mapping disabled will have no visible entities
            // so no meshes will be queued

            for entity in visible_entities.iter::<WithMesh>().copied() {
                let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(entity)
                else {
                    continue;
                };
                if !mesh_instance
                    .flags
                    .contains(RenderMeshInstanceFlags::SHADOW_CASTER)
                {
                    continue;
                }

                let Some(material_asset_id) = render_material_instances.get(&entity) else {
                    continue;
                };
                let Some(material) = render_materials.get(*material_asset_id) else {
                    continue;
                };

                let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                    continue;
                };

                let mut mesh_key =
                    light_key | MeshPipelineKey::from_bits_retain(mesh.key_bits.bits());

                mesh_key |= match material.properties.alpha_mode {
                    AlphaMode::Mask(_)
                    | AlphaMode::Blend
                    | AlphaMode::Premultiplied
                    | AlphaMode::Add
                    | AlphaMode::AlphaToCoverage => MeshPipelineKey::MAY_DISCARD,
                    _ => MeshPipelineKey::NONE,
                };

                
                let pipeline_id = pipelines.specialize(
                    &pipeline_cache,
                    &prepass_pipeline,
                    MaterialPipelineKey {
                        mesh_key,
                        bind_group_data: material.key.clone(),
                    },
                    &mesh.layout,
                );

                let pipeline_id = match pipeline_id {
                    Ok(id) => id,
                    Err(err) => {
                        error!("{}", err);
                        continue;
                    }
                };

                mesh_instance
                    .material_bind_group_id
                    .set(material.get_bind_group_id());


                shadow_phase.add(
                    ShadowBinKey {
                        draw_function: draw_shadow_mesh,
                        pipeline: pipeline_id,
                        asset_id: mesh_instance.mesh_asset_id.into(),
                    },
                    entity,
                    BinnedRenderPhaseType::UnbatchableMesh,
                );
            }
        }
    }
}