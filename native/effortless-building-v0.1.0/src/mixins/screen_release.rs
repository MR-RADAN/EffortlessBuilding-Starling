fn effortless_building_screen_release(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    use crate::game::effortless_building as m;

    if button == 0
        && screen.id == "mod"
        && game.native_mod_screen() == Some(m::ID)
        && m::release_captured_left_press()
    {
        *canceled = true;
    }
}
