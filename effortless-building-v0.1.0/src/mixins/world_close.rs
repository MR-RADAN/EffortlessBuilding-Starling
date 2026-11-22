// Retire all selection/UI state before Starling releases the active world.
// This hook runs only when the game closes a world, not when the builder panel closes.
fn effortless_building_world_close(_game: &mut crate::game::Game<'_>) {
    crate::game::effortless_building::reset_world_session();
}
