#import bevy_pbr::utils::rand_f
#import bevy_procedural_grass::grass_types::{GrassInstance, Aabb2d};

@group(0) @binding(0) var<storage, read_write> output: array<GrassInstance>;
@group(0) @binding(1) var heightmap: texture_2d<f32>;
@group(0) @binding(2) var<uniform> height_scale: f32;
@group(0) @binding(3) var<uniform> height_offset: f32;
@group(0) @binding(4) var<uniform> chunk_aabb: Aabb2d;
@group(0) @binding(5) var<uniform> aabb: Aabb2d;

@compute @workgroup_size(512)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>, 
    @builtin(local_invocation_id) local_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    var state: u32 = global_id.x + u32(chunk_aabb.min.x) * 1000u + u32(chunk_aabb.min.y) * 2000u;
    let u = rand_f(&state);
    state = state * 747796405u + 2891336453u;
    let v = rand_f(&state);
    let local_uv = vec2<f32>(u, v);

    let chunk_position = chunk_aabb.min + (local_uv * (chunk_aabb.max - chunk_aabb.min));
    
    let global_uv = (chunk_position - aabb.min) / (aabb.max - aabb.min); 

    let dimensions = textureDimensions(heightmap);
    var texture_coords = vec2<i32>(global_uv * vec2<f32>(dimensions));

    texture_coords = max(vec2<i32>(0), min(texture_coords, vec2<i32>(dimensions) - vec2<i32>(1)));

    let height = textureLoad(heightmap, texture_coords, 0).r;

    output[global_id.x] = GrassInstance(vec4<f32>(chunk_position.x, height * height_scale + height_offset, chunk_position.y, 1.0));  
}
