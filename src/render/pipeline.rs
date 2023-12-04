use bevy::{prelude::*, pbr::{MeshPipeline, MeshPipelineKey}, render::{render_resource::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, SpecializedMeshPipeline, RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat, TextureSampleType, TextureViewDimension}, renderer::RenderDevice, mesh::MeshVertexBufferLayout}};

use super::instance::GrassData;

#[derive(Resource)]
pub struct GrassPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    pub color_layout: BindGroupLayout,
    pub wind_layout: BindGroupLayout,
    pub blade_layout: BindGroupLayout,
    pub wind_map_layout: BindGroupLayout,
}

impl FromWorld for GrassPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/grass.wgsl");
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let mesh_pipeline = world.resource::<MeshPipeline>();

        let color_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("grass_color_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let wind_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("wind_layout"),
            entries: &[
                BindGroupLayoutEntry {
                binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let blade_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("blade_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let wind_map_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("light_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ]
        });

        GrassPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            color_layout,
            wind_layout,
            blade_layout,
            wind_map_layout,
        }
    }
}

impl SpecializedMeshPipeline for GrassPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<GrassData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    shader_location: 4,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 6]>() as u64,
                    shader_location: 5,
                },
            ],
        });
        descriptor.layout.push(self.color_layout.clone());
        descriptor.layout.push(self.wind_layout.clone());
        descriptor.layout.push(self.blade_layout.clone());
        descriptor.layout.push(self.wind_map_layout.clone());

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}