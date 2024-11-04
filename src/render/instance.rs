use bevy::render::render_resource::ShaderType;


#[derive(ShaderType)]
pub struct GrassInstanceData {
    position: [f32; 4],
    chunk_uv: [f32; 2],
    facing: [f32; 2],
    // tip_color: [f32; 4],
    // base_color: [f32; 4],
    length: f32,
    tilt: f32,
    midpoint: f32,
    curve: f32,
}