fn effortless_building_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use crate::game::effortless_building as m;
    if mod_id != m::ID {
        return;
    }
    let tool = m::selected_tool(|key| game.native_setting(m::ID, key));
    buttons.extend(m::setting_labels(tool));
}
