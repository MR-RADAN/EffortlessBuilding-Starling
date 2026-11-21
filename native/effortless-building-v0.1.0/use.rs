// Game::use_item HEAD: while Effortless Building is enabled, a main-hand
// right-click on a block records a corner and suppresses the held item's use.
fn effortless_building_use(
    game: &mut crate::game::Game<'_>,
) -> core::ops::ControlFlow<Result<Option<Hand>>> {
    match game.effortless_building_use() {
        Some(hand) => core::ops::ControlFlow::Break(Ok(hand)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
