@group(0) @binding(0) var<storage, read> vote_buffer: array<u32>;
@group(0) @binding(1) var<storage, read_write> scan_buffer: array<u32>;
@group(0) @binding(2) var<storage, read_write> block_sums: array<u32>;
var<workgroup> temp: array<u32, 128>;

@compute @workgroup_size(64)
fn scan(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let idx = global_id.x;
    let local_idx = local_id.x;
    let group_idx = workgroup_id.x;

    var offset = 1u;
    temp[2u * local_idx] = vote_buffer[2u * idx];
    temp[2u * local_idx + 1u] = vote_buffer[2u * idx + 1u];
    let element_count = 128u;
    var d: u32;

    for (d = element_count >> 1u; d > 0u; d >>= 1u) {
        workgroupBarrier();

        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            temp[bi] += temp[ai];
        }

        offset *= 2u;
    }

    if (local_idx == 0u) {
        block_sums[group_idx] = temp[element_count - 1];
        temp[element_count - 1u] = 0u;
    }

    for (d = 1u; d < element_count; d *= 2u) {
        offset >>= 1u;

        workgroupBarrier();
        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            let t = temp[ai];
            temp[ai] = temp[bi];
            temp[bi] += t;
        }
    }

    workgroupBarrier();

    scan_buffer[2u * idx] = temp[2u * local_idx];
    scan_buffer[2u * idx + 1u] = temp[2u * local_idx + 1u];
}