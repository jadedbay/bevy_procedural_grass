use bevy::prelude::*;

use crate::grass::chunk::GrassChunks;

pub(crate) mod aabb;

// #[allow(dead_code)]
// pub fn draw_chunks(
//     mut gizmos: Gizmos,
//     query: Query<&GrassChunks>,
// ) {
//     for chunks in query.iter() {
//         for chunk in &chunks.0 {
//             gizmos.cuboid(aabb::aabb_transform(chunk.1.aabb, GlobalTransform::IDENTITY), Color::WHITE);
//         }
//     }
// }

