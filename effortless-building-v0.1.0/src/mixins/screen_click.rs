fn effortless_building_screen_click(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
) {
    use crate::game::effortless_building as m;

    if button != 0 || screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }

    let layout = m::ui_layout(screen.rect);
    if !m::contains(layout.panel, screen.mouse) {
        return;
    }

    // The native Post hook runs after the screen's input handlers and has no
    // cancellation parameter. Record the UI action only; never claim that this
    // callback suppresses the game's underlying mouse event.
    if let Some(action) = m::ui_action_at(&layout, screen.mouse) {
        m::queue_ui_action(action);
    }
}
