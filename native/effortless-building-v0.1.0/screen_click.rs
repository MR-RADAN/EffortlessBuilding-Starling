fn effortless_building_screen_click(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    use crate::game::effortless_building as m;

    if button != 0 || screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }

    let layout = m::ui_layout(screen.rect);
    if !m::contains(layout.panel, screen.mouse) {
        return;
    }

    // Own the full left-click lifecycle inside the panel so the world cannot
    // receive a click intended for this UI.
    m::capture_left_press();
    if let Some(action) = m::ui_action_at(&layout, screen.mouse) {
        m::queue_ui_action(action);
    }
    *canceled = true;
}
