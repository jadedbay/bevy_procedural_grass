#define_import_path bevy_procedural_grass::grass_types

struct GrassInstance {
    position: vec4<f32>,
}

struct Aabb2d {
    min: vec2<f32>,
    max: vec2<f32>,
}