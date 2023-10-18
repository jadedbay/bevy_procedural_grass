use bevy::{prelude::*, pbr::{wireframe::{WireframePlugin, Wireframe}, SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey, MeshPipeline, MeshUniform}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::{RenderCommandResult, TrackedRenderPass, RenderCommand, PhaseItem, SetItemPipeline, RenderPhase, DrawFunctions, AddRenderCommand}, mesh::{GpuBufferInfo, MeshVertexBufferLayout, VertexAttributeValues}, render_asset::RenderAssets, render_resource::{VertexFormat, VertexAttribute, VertexStepMode, VertexBufferLayout, SpecializedMeshPipelineError, RenderPipelineDescriptor, SpecializedMeshPipeline, BufferUsages, BufferInitDescriptor, Buffer, PipelineCache, SpecializedMeshPipelines}, renderer::RenderDevice, view::{ExtractedView, NoFrustumCulling}, RenderApp, Render, RenderSet}, ecs::{query::QueryItem, system::{SystemParamItem, lifetimeless::{Read, SRes}}}, core_pipeline::core_3d::Transparent3d};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, prelude::ReflectInspectorOptions, InspectorOptions};
use bytemuck::{Pod, Zeroable};
use rand::Rng;

use super::terrain::Terrain;

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[inspector()]
    pub mesh: Handle<Mesh>,
    pub density: u32,
    pub material_data: InstanceMaterialData,
}

pub fn generate_grass_data(
    mut query: Query<(&Handle<Mesh>, &Terrain, &mut Grass)>,
    meshes: Res<Assets<Mesh>>
) {
    for (mesh_handle, terrain, mut grass) in query.iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            grass.material_data = vertices_as_material_data(mesh);
        }
    }
}

pub fn spread_grass_blades(
    mut commands: Commands,
    mut query: Query<(&Handle<Mesh>, &Transform, &Terrain, &mut Grass, Option<&Children>), Or<(Changed<Terrain>, Changed<Transform>)>>,
    meshes: Res<Assets<Mesh>>
) {
    for (mesh_handle, transform, terrain, mut grass, children) in query.iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            let mut data = Vec::new();

            let size = transform.scale / 2.0;

            for x in -size.x as i32..=size.x as i32 {
                for z in -size.y as i32..=size.y as i32 {
                    data.push(InstanceData {
                        position: Vec3::new(x as f32 / transform.scale.x, 0.0, z as f32/ transform.scale.y),
                        scale: 0.1,
                        color: Color::GREEN.into(),
                    });
                }
            }

            grass.material_data = InstanceMaterialData(data);

            update_grass_object(&mut commands, &grass.material_data, &children);
        }
    }
}


fn vertices_as_material_data(mesh: &Mesh) -> InstanceMaterialData {
    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        return InstanceMaterialData(
            positions.iter()
                .map(|&position| InstanceData {
                    position: Vec3::new(position[0], position[1], position[2]),
                    scale: 0.1,
                    color: Color::GREEN.into(),
                })
                .collect(),
        );
    }

    InstanceMaterialData(Vec::new())
}

fn update_grass_object(commands: &mut Commands, data: &InstanceMaterialData, children: &Option<&Children>) {
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(*child).insert(data.clone());
        }
    }
}

pub fn spawn_grass(
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


pub fn update_grass_material_data(
    mut commands: Commands,
    mut query: Query<(Entity, &Handle<Mesh>, &Terrain, &mut Grass, Option<&Children>), Changed<Terrain>>,
    meshes: Res<Assets<Mesh>>
) {
    for (entity, mesh_handle, terrain, mut grass, children) in query.iter_mut() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                grass.material_data = InstanceMaterialData(
                    positions.iter()
                        .map(|&position| InstanceData {
                            position: Vec3::new(position[0], position[1], position[2]),
                            scale: 0.1,
                            color: Color::GREEN.into(),
                        })
                        .collect(),
                );
            }
        }

        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(*child).insert(grass.material_data.clone());
            }
        }
    }
}

#[derive(Component, Deref, Clone, Reflect)]
pub struct InstanceMaterialData(Vec<InstanceData>);

impl Default for InstanceMaterialData {
    fn default() -> Self {
        Self(Vec::new())
    }
}

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

#[derive(Clone, Copy, Pod, Zeroable, Reflect)]
#[repr(C)]
pub struct InstanceData {
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