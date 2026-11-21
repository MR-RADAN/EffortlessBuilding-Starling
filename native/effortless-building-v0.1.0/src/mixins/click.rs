fn effortless_building_settings_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    use crate::game::effortless_building as m;
    if mod_id != m::ID {
        return;
    }

    if button < m::TOOLS.len() {
        game.native_set_setting(m::ID, "tool", &button.to_string());
    } else if button == m::TOOLS.len() {
        let _ = game.native_mod_screen_open(m::ID);
    }
}
