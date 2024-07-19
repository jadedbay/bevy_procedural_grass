struct GrassInstanceData {
    position: vec4<f32>,
    normal: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> instance_data: array<GrassInstanceData>;
@group(0) @binding(1) var<storage, read> vote: array<u32>;
@group(0) @binding(2) var<storage, read_write> scan_array: array<u32>;
@group(0) @binding(3) var<storage, read_write> compact_array: array<GrassInstanceData>;

var<workgroup> temp: array<u32, 64>;

@compute @workgroup_size(64)
fn scan(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let idx = global_id.x;
    let local_idx = local_id.x;

    // Load data into shared memory
    temp[local_idx] = vote[idx];
    workgroupBarrier();

    // Up-sweep (reduce) phase
    var offset = 1u;
    for (var d = 64u >> 1u; d > 0u; d = d >> 1u) {
        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            temp[bi] = temp[bi] + temp[ai];
        }
        offset = offset * 2u;
        workgroupBarrier();
    }

    // Clear the last element
    if (local_idx == 0u) {
        temp[63] = 0u;
    }
    workgroupBarrier();

    // Down-sweep phase
    for (var d = 1u; d < 64u; d = d * 2u) {
        offset = offset >> 1u;
        if (local_idx < d) {
            let ai = offset * (2u * local_idx + 1u) - 1u;
            let bi = offset * (2u * local_idx + 2u) - 1u;
            let t = temp[ai];
            temp[ai] = temp[bi];
            temp[bi] = temp[bi] + t;
        }
        workgroupBarrier();
    }

    // Write results to scan_array
    scan_array[idx] = temp[local_idx];
}

@compute @workgroup_size(64)
fn compact(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let idx = global_id.x;
    if (vote[idx] == 1u) {
        let new_idx = scan_array[idx];
        compact_array[new_idx] = instance_data[idx];
    }
}