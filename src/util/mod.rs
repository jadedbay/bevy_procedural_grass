use bevy::prelude::*;

use crate::grass::Grass;

pub(crate) mod aabb;

#[allow(dead_code)]
pub fn draw_chunks(
    mut gizmos: Gizmos,
    query: Query<&Grass>,
) {
    for grass in query.iter() {
        for chunk in &grass.chunks {
            gizmos.cuboid(aabb::aabb_transform(chunk.1.aabb, GlobalTransform::IDENTITY), Color::WHITE);
        }
    }
}