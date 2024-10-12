use bevy::render::render_resource::ShaderType;


#[derive(ShaderType)]
pub struct GrassInstanceData {
    position: [f32; 4],
    chunk_uv: [f32; 2],
    facing: [f32; 2],
}