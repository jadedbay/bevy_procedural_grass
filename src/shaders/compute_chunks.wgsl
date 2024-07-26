#import bevy_procedural_grass::grass_types::Aabb;

struct Counts {
    instance_count: u32,
    workgroup_count: u32,
    scan_workgroup_count: u32,
    scan_groups_workgroup_count: u32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;

@group(1) @binding(0) var<uniform> aabb: Aabb;
@group(1) @binding(1) var<storage, read_write> indices_index: array<u32>;
@group(1) @binding(2) var<storage, read_write> counts: Counts;

@compute @workgroup_size(32)
fn main(
    
) {

}