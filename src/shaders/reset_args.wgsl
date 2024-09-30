#import bevy_procedural_grass::grass_types::DrawIndexedIndirectArgs;

@group(0) @binding(0) var<storage, read_write> indirect_args: DrawIndexedIndirectArgs;

@compute @workgroup_size(1)
fn reset_args() {
    indirect_args.instance_count = 0u;
}