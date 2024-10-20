use bevy::{pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline}, prelude::*, render::{globals::GlobalsUniform, mesh::MeshVertexBufferLayoutRef, render_asset::{RenderAssetUsages, RenderAssets}, render_resource::{AsBindGroup, AsBindGroupShaderType, BindGroupLayout, Extent3d, RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError, TextureDimension, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode}, texture::GpuImage}};
use noise::{NoiseFn, Perlin, Simplex};

use crate::render::instance::GrassInstanceData;

pub type GrassMaterial = ExtendedMaterial<StandardMaterial, GrassMaterialExtension>;

#[derive(Asset, AsBindGroup, Reflect, Clone, Default)]
#[uniform(100, GrassMaterialUniform)]
#[reflect(Default)]
pub struct GrassMaterialExtension {
    pub tip_color: Color,
    pub width: f32,

    pub curve: f32,
    pub tilt: f32,
    pub midpoint: f32,

    pub roughness_variance: f32,
    pub reflectance_variance: f32,
    pub min_ao: f32,

    pub midrib_softness: f32,
    pub rim_position: f32,
    pub rim_softness: f32,
    pub width_normal_strength: f32,

    pub texture_strength: f32,
    #[texture(101)] pub texture: Option<Handle<Image>>,

    pub oscillation_speed: f32,
    pub oscillation_flexibility: f32,
    pub oscillation_strength: f32,
    #[texture(102)] pub wind_texture: Handle<Image>,
                                                    }
impl MaterialExtension for GrassMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_procedural_grass/shaders/grass_vertex.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_procedural_grass/shaders/grass_fragment.wgsl".into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_procedural_grass/shaders/grass_vertex.wgsl".into()
    }
    
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> { 
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<GrassInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: VertexFormat::Float32x4.size() + VertexFormat::Float32x2.size(),
                    shader_location: 5,
                }
            ],
        });
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct GrassMaterialUniform {
    pub tip_color: Vec4,
    pub width: f32,
    pub curve: f32,
    pub tilt: f32,
    pub midpoint: f32,
    pub roughness_variance: f32,
    pub reflectance_variance: f32,
    pub min_ao: f32,
    pub midrib_softness: f32,
    pub rim_position: f32,
    pub rim_softness: f32,
    pub width_normal_strength: f32,
    pub texture_strength: f32, 
    pub oscillation_speed: f32,
    pub oscillation_flexibility: f32,
    pub oscillation_strength: f32,
}

impl AsBindGroupShaderType<GrassMaterialUniform> for GrassMaterialExtension {
    fn as_bind_group_shader_type(
        &self, 
        _images: &RenderAssets<GpuImage>
    ) -> GrassMaterialUniform {
        GrassMaterialUniform {
            tip_color: LinearRgba::from(self.tip_color).to_vec4(),
            width: self.width,
            curve: self.curve,
            tilt: self.tilt,
            midpoint: self.midpoint,
            roughness_variance: self.roughness_variance,
            reflectance_variance: self.reflectance_variance,
            min_ao: self.min_ao,
            midrib_softness: self.midrib_softness,
            rim_position: self.rim_position,
            rim_softness: self.rim_softness,
            width_normal_strength: self.width_normal_strength,
            texture_strength: self.texture_strength,
            oscillation_speed: self.oscillation_speed,
            oscillation_flexibility: self.oscillation_flexibility,
            oscillation_strength: self.oscillation_strength,
        }
    }
}

pub fn create_grass_texture(
    width: u32,
    height: u32,
    frequency: [f64; 2],
) -> Image {
    let simplex = Simplex::new(0);
    let mut texture_data = vec![0; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = (y as f64 / height as f64) / 1.0;
            let noise_value = (simplex.get([nx * frequency[0], ny * frequency[1]]) + 1.0) / 2.0;
            let index = ((y * width + x) * 4) as usize;

            texture_data[index] = (noise_value * 255.0) as u8;     // R
            texture_data[index + 1] = (noise_value * 255.0) as u8; // G
            texture_data[index + 2] = (noise_value * 255.0) as u8; // B
            texture_data[index + 3] = 255;
        }
    }

    let texture = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    texture
}