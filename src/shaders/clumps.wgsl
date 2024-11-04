#import bevy_procedural_grass::GrassClump;

@group(0) @binding(0) var<uniform> clump_size: vec2<f32>;
@group(0) @binding(1) var<storage, read_write>: array<GrassClump>;
 
@group(0) @binding(0) var<uniform> chunk_aabb: Aabb2d;

fn main() {

}