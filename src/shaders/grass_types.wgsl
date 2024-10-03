#define_import_path bevy_procedural_grass::grass_types

struct GrassInstance {
    position: vec4<f32>,
}

struct Aabb2d {
    min: vec2<f32>,
    max: vec2<f32>,
}

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
}

struct GrassMaterial {
    facing_angle: f32,
    curve: f32,
}