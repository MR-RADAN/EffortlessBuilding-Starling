// Pair a cancelled builder-panel press with its release so screen drag
// handling cannot receive only the second half of the mouse gesture.
fn effortless_building_screen_release_pre(button: i32, canceled: &mut bool) {
    use std::sync::atomic::Ordering;

    if button == 0
        && EFFORTLESS_BUILDING_LEFT_PRESS_CANCELLED.swap(false, Ordering::AcqRel)
    {
        *canceled = true;
    }
}
