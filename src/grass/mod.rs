pub mod grass;
pub mod wind;
pub mod chunk;
pub mod mesh;

// pub struct GrassPlugin;

// impl Plugin for GrassPlugin {
//     fn build(&self, app: &mut App) {
//         app.register_type::<Grass>()
//         .add_systems(PostStartup, grass::load_grass)
//         .add_systems(Update, (grass::update_grass_data, grass::update_grass_params, chunk::grass_culling))
//         .init_asset::<GrassInstanceData>()
//         .add_plugins(RenderAssetPlugin::<GrassInstanceData>::default())
//         .add_plugins(ExtractComponentPlugin::<GrassColorData>::default())
//         .add_plugins(ExtractComponentPlugin::<WindData>::default())
//         .add_plugins(ExtractComponentPlugin::<BladeData>::default())
//         .add_plugins(ExtractComponentPlugin::<GrassToDraw>::default())
//         .add_plugins(ExtractComponentPlugin::<WindMap>::default());

//         let render_app = app.sub_app_mut(RenderApp);
//         render_app.add_render_command::<Opaque3d, DrawGrass>()
//         .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
//         .add_systems(
//             Render,
//             (
//                 queue::grass_queue.in_set(RenderSet::QueueMeshes),
//                 prepare::prepare_color_buffers.in_set(RenderSet::PrepareBindGroups),
//                 prepare::prepare_wind_buffers.in_set(RenderSet::PrepareBindGroups),
//                 prepare::prepare_blade_buffers.in_set(RenderSet::PrepareBindGroups),
//                 prepare::prepare_wind_map_buffers.in_set(RenderSet::PrepareBindGroups),
//             ),
//         );
//     }

//     fn finish(&self, app: &mut App) {
//         app.sub_app_mut(RenderApp)
//             .init_resource::<GrassPipeline>();
//     }
// }