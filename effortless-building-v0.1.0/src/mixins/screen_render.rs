fn effortless_building_screen_render(
    game: &crate::game::Game<'_>,
    batch: &mut crate::mod_api::HudBatch,
    screen: &crate::mod_api::NativeScreenView<'_>,
) {
    crate::game::effortless_building::draw_screen(game, batch, screen);
}
