#define_import_path bevy_procedural_grass::grass_types

struct GrassInstance {
    position: vec4<f32>,
    normal: vec4<f32>,
}

#ifdef SCAN
@group(0) @binding(0) var<storage, read> vote_buffer: array<u32>;
@group(0) @binding(1) var<storage, read_write> scan_buffer: array<u32>;
@group(0) @binding(2) var<storage, read_write> block_sums: array<u32>;
var<workgroup> temp: array<u32, 128>;
#endif