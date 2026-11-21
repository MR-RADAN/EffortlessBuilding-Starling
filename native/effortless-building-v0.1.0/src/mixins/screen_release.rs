fn effortless_building_screen_release(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
) {
    // Mouse release Post has no cancellation argument in Starling 0.9.0.
    // Keep this hook observational; it must not claim to consume the event.
    if button == 0
        && screen.id == "mod"
        && game.native_mod_screen().is_none()
    {
        return;
    }
}
