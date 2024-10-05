use bevy::{math::bounding::Aabb2d, prelude::*, render::{primitives::Aabb, render_resource::ShaderType}};

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub(crate) struct Aabb2dGpu {
    min: Vec2,
    max: Vec2,
}

impl From<Aabb2d> for Aabb2dGpu {
    fn from(aabb: Aabb2d) -> Self {
        Self {
            min: aabb.min,
            max: aabb.max,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn aabb_transform(aabb: Aabb, transform: GlobalTransform) -> GlobalTransform {
    transform * GlobalTransform::from(
        Transform::from_translation(aabb.center.into())
            .with_scale((aabb.half_extents * 2.).into()),
    )
}