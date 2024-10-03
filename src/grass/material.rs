use bevy::{pbr::{ExtendedMaterial, MaterialExtension}, prelude::*, render::{render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, ShaderRef, SpecializedMeshPipelineError, TextureDimension, TextureFormat}}};
use noise::{NoiseFn, Perlin, Simplex};

pub type GrassMaterial = ExtendedMaterial<StandardMaterial, GrassMaterialExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct GrassMaterialExtension {
    #[uniform(100)]
    pub width: f32,
    #[uniform(100)]
    pub curve: f32,
    #[uniform(100)]
    pub roughness_variance: f32,
    #[uniform(100)]
    pub reflectance_variance: f32,
    #[uniform(100)]
    pub midrib_softness: f32,
    #[uniform(100)]
    pub rim_position: f32,
    #[uniform(100)]
    pub rim_softness: f32,
    #[uniform(100)]
    pub width_normal_strength: f32,
    #[texture(101)]
    pub texture: Option<Handle<Image>>, // Create texture binding in material extension instead of using base_color_texture in StandardMaterial to customize how its applied. 
                                        // Could just use StandardMaterial texture if I can work out how to disable StandardMaterialFlags::BASE_COLOR_TEXTURE
}
impl MaterialExtension for GrassMaterialExtension {}

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