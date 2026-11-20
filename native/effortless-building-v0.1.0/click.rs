fn effortless_building_settings_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    use crate::game::effortless_building as m;
    if mod_id != m::ID || button >= m::TOOLS.len() {
        return;
    }
    game.native_set_setting(m::ID, "tool", &button.to_string());
}
