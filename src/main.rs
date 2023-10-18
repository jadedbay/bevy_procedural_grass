use bevy::{prelude::*, pbr::{wireframe::{WireframePlugin, Wireframe}, SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey, MeshPipeline, MeshUniform}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::{RenderCommandResult, TrackedRenderPass, RenderCommand, PhaseItem, SetItemPipeline, RenderPhase, DrawFunctions, AddRenderCommand}, mesh::{GpuBufferInfo, MeshVertexBufferLayout, VertexAttributeValues}, render_asset::RenderAssets, render_resource::{VertexFormat, VertexAttribute, VertexStepMode, VertexBufferLayout, SpecializedMeshPipelineError, RenderPipelineDescriptor, SpecializedMeshPipeline, BufferUsages, BufferInitDescriptor, Buffer, PipelineCache, SpecializedMeshPipelines}, renderer::RenderDevice, view::{ExtractedView, NoFrustumCulling}, RenderApp, Render, RenderSet}, ecs::{query::QueryItem, system::{SystemParamItem, lifetimeless::{Read, SRes}}}, core_pipeline::core_3d::Transparent3d};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;
use bytemuck::{Pod, Zeroable};
use procedural_grass::{grass::{CustomMaterialPlugin, Grass, InstanceMaterialData, InstanceData}, GrassPlugin, terrain::TerrainPlugin};
use procedural_grass::terrain::Terrain;

pub mod procedural_grass;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        TerrainPlugin,
        GrassPlugin,
        CustomMaterialPlugin,
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        PbrBundle {
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
            ..Default::default()
        }, 
        Wireframe,
        Terrain::default(),
        Grass {
            mesh: asset_server.load::<Mesh, &str>("meshes/grass_blade.glb#Mesh0/Primitive0"),
            density: 5,
            material_data: InstanceMaterialData::default()
        }
    ));
     
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

