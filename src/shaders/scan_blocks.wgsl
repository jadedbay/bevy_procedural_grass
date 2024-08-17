var<push_constant> group_count: u32;
@group(0) @binding(0) var<storage, read> block_sums_in: array<u32>;
@group(0) @binding(1) var<storage, read_write> block_sums_out: array<u32>;
var<workgroup> temp_blocks: array<u32, 2048>;

@compute @workgroup_size(1024)
fn scan_blocks(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let idx = global_id.x;
    let local_idx = local_id.x;
    let group_idx = workgroup_id.x;

    var offset = 1u;
    temp_blocks[2u * local_idx] = block_sums_in[2u * idx];
    temp_blocks[2u * local_idx + 1u] = block_sums_in[2u * idx + 1u];
    var d: u32;

    for (d = group_count >> 1u; d > 0u; d >>= 1u) {
        workgroupBarrier();

        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            temp_blocks[bi] += temp_blocks[ai];
        }

        offset *= 2u;
    }

    if (idx == 0u) {
        temp_blocks[group_count - 1u] = 0u;
    }

    for (d = 1u; d < group_count; d *= 2u) {
        offset >>= 1u;

        workgroupBarrier();
        if (idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            let t = temp_blocks[ai];
            temp_blocks[ai] = temp_blocks[bi];
            temp_blocks[bi] += t;
        }
    }

    workgroupBarrier();

    block_sums_out[2u * idx] = temp_blocks[2u * idx];
    block_sums_out[2u * idx + 1u] = temp_blocks[2u * idx + 1u];
}
