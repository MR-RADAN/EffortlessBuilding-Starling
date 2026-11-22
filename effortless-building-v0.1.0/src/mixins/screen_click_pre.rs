// Cancel only clicks owned by the builder panel, before vanilla screen dispatch.
fn effortless_building_screen_click_pre(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::ScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    use crate::game::effortless_building as m;
    use std::sync::atomic::Ordering;

    if button != 0 {
        return;
    }

    // Every new left press supersedes a stale press whose release may have
    // been lost during a screen transition.
    m::LEFT_PRESS_CANCELLED.store(false, Ordering::Release);

    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }

    let layout = m::ui_layout(screen.rect);
    if !m::contains(layout.panel, screen.mouse) {
        return;
    }

    // Own the panel surface so clicks cannot leak through to underlying slots.
    // Only actual controls enqueue actions; panel background is a safe no-op.
    if let Some(action) = m::ui_action_at(&layout, screen.mouse) {
        m::queue_ui_action(action);
    }
    *canceled = true;
    EFFORTLESS_BUILDING_LEFT_PRESS_CANCELLED.store(true, Ordering::Release);
}
