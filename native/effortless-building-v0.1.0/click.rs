fn effortless_building_settings_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    use crate::game::effortless_building as m;
    if mod_id != m::ID || button != 0 {
        return;
    }
    let current = m::selected_tool(|key| game.native_setting(m::ID, key));
    let next = (current + 1) % m::TOOLS.len();
    game.native_set_setting(m::ID, "tool", &next.to_string());
}
