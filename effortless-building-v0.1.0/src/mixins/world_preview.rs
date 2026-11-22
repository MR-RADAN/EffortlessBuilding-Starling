// World-space selection outline and bounded, spatially distributed preview markers.
// This is visualization only: it does not read or modify block states.
fn effortless_building_preview(
    game: &crate::game::Game<'_>,
    view: &crate::mod_api::WorldRenderView,
    draw: &mut horizon_client::world_geometry::WorldGeometry,
) {
    game.effortless_building_preview(view, draw);
}
