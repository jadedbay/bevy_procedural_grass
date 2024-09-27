#import bevy_pbr::utils::rand_f
#import bevy_procedural_grass::grass_types::GrassInstance;

@group(0) @binding(0) var<storage, read_write> output: array<GrassInstance>;
@group(0) @binding(1) var heightmap: texture_2d<f32>;
//@group(0) @binding(2) var<uniform> chunk_pos: vec3<u32>;

@compute @workgroup_size(512)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>, 
    @builtin(local_invocation_id) local_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    var state: u32 = global_id.x;
    let u = rand_f(&state);
    state = state * 747796405u + 2891336453u;
    let v = rand_f(&state);
    let uv = vec2<f32>(u, v);

    let position = (uv * 100.0) - 50.0;

    let dimensions = textureDimensions(heightmap);
    let height = textureLoad(heightmap, vec2<i32>(uv * vec2<f32>(dimensions)), 0).r;

    output[global_id.x] = GrassInstance(vec4<f32>(position.x, height * 6.0 - 0.1, position.y, 1.0));  
}
