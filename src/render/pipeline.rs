use bevy::{prelude::*, pbr::{MeshPipeline, MeshPipelineKey}, render::{render_resource::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, SpecializedMeshPipeline, RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat, TextureSampleType, TextureViewDimension}, renderer::RenderDevice, mesh::MeshVertexBufferLayout}};

use crate::GRASS_SHADER_HANDLE;

use super::instance::GrassData;

#[derive(Resource)]
pub struct GrassPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    pub grass_layout: BindGroupLayout,
    pub wind_layout: BindGroupLayout,
    pub displacement_layout: BindGroupLayout,
}

impl FromWorld for GrassPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let mesh_pipeline = world.resource::<MeshPipeline>();

        let grass_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("grass_layout"),
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
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
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
                },
                BindGroupLayoutEntry {
                    binding: 1,
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

        let displacement_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("displacement_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ]
        });

        GrassPipeline {
            shader: GRASS_SHADER_HANDLE,
            mesh_pipeline: mesh_pipeline.clone(),
            grass_layout,
            wind_layout,
            displacement_layout,
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
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 6]>() as u64,
                    shader_location: 5,
                },
            ],
        });
        descriptor.layout.push(self.grass_layout.clone());
        descriptor.layout.push(self.wind_layout.clone());
        descriptor.layout.push(self.displacement_layout.clone());

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}