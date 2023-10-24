use bevy::{prelude::*, pbr::{SetMeshBindGroup, SetMeshViewBindGroup, MeshPipelineKey, MeshUniform}, render::{extract_component::{ExtractComponent, ExtractComponentPlugin}, render_phase::{RenderCommandResult, TrackedRenderPass, RenderCommand, PhaseItem, SetItemPipeline, RenderPhase, DrawFunctions, AddRenderCommand}, render_asset::RenderAssets, render_resource::{BufferUsages, BufferInitDescriptor, Buffer, PipelineCache, SpecializedMeshPipelines, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferBinding, BindGroup}, renderer::RenderDevice, view::{ExtractedView, NoFrustumCulling}, RenderApp, Render, RenderSet, mesh::GpuBufferInfo}, ecs::{query::QueryItem, system::{SystemParamItem, lifetimeless::{Read, SRes}}}, core_pipeline::core_3d::Opaque3d};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use bytemuck::{Pod, Zeroable};
use noise::NoiseFn;
use rand::Rng;

use crate::terrain::component::Terrain;

use super::pipeline::CustomPipeline;

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub material_data: InstanceMaterialData,
    pub density: u32,
    pub color: GrassColor,
    pub regenerate: bool,
}

pub fn update_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Terrain, &mut Grass)>,
    grass_entity_query: Query<(Entity, &GrassId)>,
) {
    for (entity, transform, terrain, mut grass) in query.iter_mut() {
        if grass.regenerate {
            generate_grass_data(transform, terrain, &mut grass);
            for (grass_entity, grass_id) in grass_entity_query.iter() {
                if grass_id.0 == entity.index() {
                    commands.entity(grass_entity).insert(grass.material_data.clone());
                }
            }

            grass.regenerate = false;
        }
    }
}

pub fn update_grass_color(
    mut commands: Commands,
    query: Query<(Entity, &Grass), Changed<Grass>>,
    grass_entity_query: Query<(Entity, &GrassId)>,
) {
    for (entity, grass) in query.iter() {
        for (grass_entity, grass_id) in grass_entity_query.iter() {
            if grass_id.0 == entity.index() {
                commands.entity(grass_entity).insert(GrassColorData::from(grass.color.clone()));
            }
        }
    }
}

pub fn load_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Terrain, &mut Grass)>
) {
    for (entity, transform, terrain, mut grass) in query.iter_mut() {
        generate_grass_data(transform, terrain, &mut grass);
        spawn_grass(&mut commands, entity, &grass);
    }
}

pub fn generate_grass_data(
    transform: &Transform,
    terrain: &Terrain, 
    grass: &mut Grass,
) {
    let size = transform.scale / 2.0;
    let density = grass.density;

    let rng = rand::thread_rng();

    let data: Vec<InstanceData> = 
    (-size.x as i32 * density as i32..=size.x as i32 * density as i32)
    .flat_map(|x| {
        let mut rng = rng.clone();
        (-size.z as i32 * density as i32..=size.z as i32 * density as i32)
        .map(move |z| {
            let offset_x = rng.gen_range(-0.5..0.5);
            let offset_z = rng.gen_range(-0.5..0.5);

            let mut y = 1.;
            if let Some(noise) = &terrain.noise {
                y += (noise::Perlin::new(noise.seed).get([(((x as f32 + offset_x) / density as f32) * noise.intensity / transform.scale.x) as f64, (((z as f32 + offset_z) / density as f32) * noise.intensity / transform.scale.z) as f64]) as f32) * terrain.get_height_scale();
            }

            InstanceData {
                position: Vec3::new((x as f32 + offset_x) / density as f32, y, (z as f32 + offset_z) / density as f32),
            }
        })
    })
    .collect();
    dbg!(data.len());

    grass.material_data = InstanceMaterialData(data);
}

pub fn spawn_grass(
    commands: &mut Commands,
    entity: Entity,
    grass: &Grass,
) {
    commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        grass.material_data.clone(),
        GrassColorData::from(grass.color),
        NoFrustumCulling,
        GrassId(entity.index())
    ));
}

#[derive(Component)]
pub struct GrassId(u32);

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct GrassColor {
    ao: Color,
    color_1: Color,
    color_2: Color,
    tip: Color,
}

impl Default for GrassColor {
    fn default() -> Self {
        Self {
            ao: [0.01, 0.02, 0.05, 1.0].into(),
            color_1: [0.33, 0.57, 0.29, 1.0].into(),
            color_2: [0.08, 0.43, 0.29, 1.0].into(),
            tip: [1.0, 1.0, 1.0, 1.0].into(),
        }
    }
}


#[derive(Component, Clone, Copy, Pod, Zeroable, Reflect, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
#[repr(C)]
pub struct GrassColorData {
    ao: [f32; 4],
    color_1: [f32; 4],
    color_2: [f32; 4],
    tip: [f32; 4],
}

impl From<GrassColor> for GrassColorData {
    fn from(color: GrassColor) -> Self {
        Self {
            ao: color.ao.into(),
            color_1: color.color_1.into(),
            color_2: color.color_2.into(),
            tip: color.tip.into(),
        }
    }
}

impl ExtractComponent for GrassColorData {
    type Query = &'static GrassColorData;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Option<Self> {
        Some(item.clone())
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
        app.add_plugins(ExtractComponentPlugin::<GrassColorData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::Queue),
                    prepare_instance_buffers.in_set(RenderSet::Prepare),
                    prepare_color_buffers.in_set(RenderSet::Prepare),
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
}

fn queue_custom(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<InstanceMaterialData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_custom = opaque_3d_draw_functions.read().id::<DrawCustom>();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut opaque_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                opaque_phase.add(Opaque3d {
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

#[derive(Component)]
pub struct ColorBindGroup {
    bind_group: BindGroup,
}

fn prepare_color_buffers(
    mut commands: Commands,
    pipeline: Res<CustomPipeline>,
    query: Query<(Entity, &GrassColorData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, color) in &query {
        let layout = pipeline.color_layout.clone();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("color buffer"),
            contents: bytemuck::cast_slice(&[color.clone()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("grass color bind group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                })
            }],
        });

        commands.entity(entity).insert(ColorBindGroup {
            bind_group,
        });
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetColorBindGroup<2>,
    DrawMeshInstanced,
);

pub struct SetColorBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetColorBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Option<Read<ColorBindGroup>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        bind_group: Option<&'w ColorBindGroup>,
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