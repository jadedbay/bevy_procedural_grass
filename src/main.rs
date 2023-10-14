use bevy::{prelude::*, pbr::{wireframe::{WireframePlugin, Wireframe}, SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey, MeshPipeline, MeshUniform}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::{RenderCommandResult, TrackedRenderPass, RenderCommand, PhaseItem, SetItemPipeline, RenderPhase, DrawFunctions, AddRenderCommand}, mesh::{GpuBufferInfo, MeshVertexBufferLayout}, render_asset::RenderAssets, render_resource::{VertexFormat, VertexAttribute, VertexStepMode, VertexBufferLayout, SpecializedMeshPipelineError, RenderPipelineDescriptor, SpecializedMeshPipeline, BufferUsages, BufferInitDescriptor, Buffer, PipelineCache, SpecializedMeshPipelines}, renderer::RenderDevice, view::{ExtractedView, NoFrustumCulling}, RenderApp, Render, RenderSet}, ecs::{query::QueryItem, system::{SystemParamItem, lifetimeless::{Read, SRes}}}, core_pipeline::core_3d::Transparent3d};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_flycam::*;
use bytemuck::{Pod, Zeroable};
use terrain::{TerrainPlugin, Terrain};

pub mod terrain;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        TerrainPlugin,
        CustomMaterialPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(PostStartup, (generate_grass_data, spawn_grass))
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
            material_data: InstanceMaterialData(
                (1..=10)
                    .flat_map(|x| (1..=10).map(move |z| (x as f32 / 10.0, z as f32 / 10.0)))
                    .map(|(x, z)| InstanceData {
                        position: Vec3::new(x * 4.0 - 2.0, 0.0, z * 4.0 - 2.0),
                        scale: 0.1,
                        color: Color::GREEN.into(),
                    })
                    .collect(),
            )
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

#[derive(Component)]
struct Grass {
    mesh: Handle<Mesh>,
    material_data: InstanceMaterialData,
}

fn generate_grass_data(
    mut query: Query<(&Terrain, &mut Grass)> 
) {
    for (terrain, mut grass) in query.iter_mut() {
        grass.material_data = InstanceMaterialData(
            (1..=terrain.subdivisions + 1)
                .flat_map(|x| (1..=terrain.subdivisions + 1).map(move |z| (x as f32 / 10.0, z as f32 / 10.0)))
                .map(|(x, z)| InstanceData {
                    position: Vec3::new(x * 4.0 - 2.0, 0.0, z * 4.0 - 2.0),
                    scale: 0.1,
                    color: Color::GREEN.into(),
                })
                .collect(),
        )
    }
}

fn spawn_grass(
    mut commands: Commands,
    query: Query<(Entity, &Grass), With<Terrain>>,
) {
    for (entity, grass) in query.iter() {
        commands.entity(entity).with_children(|parent| { 
            parent.spawn((
                grass.mesh.clone(),
                SpatialBundle::INHERITED_IDENTITY,
                grass.material_data.clone(),
                NoFrustumCulling,
            ));
        });
    }
}

#[derive(Component, Deref, Clone)]
struct InstanceMaterialData(Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::Queue),
                    prepare_instance_buffers.in_set(RenderSet::Prepare),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct InstanceData {
    position: Vec3,
    scale: f32,
    color: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<InstanceMaterialData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        // meshes typically live in bind group 2. because we are using bindgroup 1
        // we need to add MESH_BINDGROUP_1 shader def so that the bindings are correctly
        // linked in the shader
        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

pub struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = SRes<RenderAssets<Mesh>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>, Read<InstanceBuffer>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        (mesh_handle, instance_buffer): (&'w Handle<Mesh>, &'w InstanceBuffer),
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed => {
                pass.draw(0..gpu_mesh.vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}