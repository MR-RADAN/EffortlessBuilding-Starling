// Starling 0.9.0 compatibility hook: claim the held-item use only when
// the selection wand captured a valid block target.
fn effortless_building_use(
    game: &mut crate::game::Game<'_>,
    _stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    if game.effortless_building_use().is_some() {
        *claimed = true;
    }
}
