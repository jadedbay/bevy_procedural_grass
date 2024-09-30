use bevy::{prelude::*, render::primitives::Aabb};

#[allow(dead_code)]
pub(crate) fn aabb_transform(aabb: Aabb, transform: GlobalTransform) -> GlobalTransform {
    transform * GlobalTransform::from(
        Transform::from_translation(aabb.center.into())
            .with_scale((aabb.half_extents * 2.).into()),
    )
}