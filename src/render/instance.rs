use bevy::render::render_resource::ShaderType;


#[derive(ShaderType)]
pub struct GrassInstanceData {
    position: [f32; 4],
}