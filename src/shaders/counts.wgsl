struct ComputeIndirectArgs {
  x: u32,
  y: u32,
  z: u32,
}

@group(0) @binding(0) var<storage, read> triangle_dispatch_count: array<u32>;
@group(0) @binding(1) var<storage, read_write> indices_index: array<u32>,
@group(0) @binding(1) var<storage, read_write> indirect_args: ComputeIndirectArgs;

@compute @workgroup_size(1)
fn main(
  @builtin(global_invocation_id) global_id: vec3<u32>,
) {
  
}

// CALCULATE ONLY THE AREA IN THE OTHER SHADER
