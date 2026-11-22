// A cancelled press must have its matching release cancelled as well.
fn effortless_building_screen_release_pre(
    _screen: &crate::game::native_hooks::ScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    use std::sync::atomic::Ordering;

    if button == 0
        && EFFORTLESS_BUILDING_LEFT_PRESS_CANCELLED.swap(false, Ordering::AcqRel)
    {
        *canceled = true;
    }
}
