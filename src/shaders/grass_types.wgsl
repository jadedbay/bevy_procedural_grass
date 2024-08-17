#define_import_path bevy_procedural_grass::grass_types

struct GrassInstance {
    position: vec4<f32>,
    normal: vec4<f32>,
}

struct Aabb {
    min: vec3<f32>,
    _padding: f32,
    max: vec3<f32>,
    _2padding: f32,
}
