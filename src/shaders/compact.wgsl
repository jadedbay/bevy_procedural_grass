#import bevy_procedural_grass::grass_types::{GrassInstance, DrawIndexedIndirectArgs};

@group(0) @binding(0) var<storage, read> instance_data: array<GrassInstance>;
@group(0) @binding(1) var<storage, read> vote_buffer: array<u32>;
@group(0) @binding(2) var<storage, read> scan_buffer: array<u32>;
@group(0) @binding(3) var<storage, read> block_sums: array<u32>;
@group(0) @binding(4) var<storage, read_write> compact_array: array<GrassInstance>;
@group(0) @binding(5) var<storage, read_write> indirect_args: DrawIndexedIndirectArgs;

@compute @workgroup_size(128)
fn compact(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
) {
    let idx = global_id.x;
    let group_idx = workgroup_id.x;
    var block_sum: u32;
    if (group_idx > 0u) {
        block_sum = block_sums[group_idx];
    }

    if (vote_buffer[idx] == 1u) {
        atomicAdd(&indirect_args.instance_count, 1u);
        compact_array[scan_buffer[idx] + block_sum] = instance_data[idx];
    }
}