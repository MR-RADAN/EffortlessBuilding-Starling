# Native UI milestone — Starling 0.8.0

Source reviewed: Starling 0.8.0 ZIP at Horizon revision
`ac874422ed8f892505ff73d67ae486ecfee7a516`.

## Exact integration points

- `native_settings_click`: settings button index 4 calls
  `native_mod_screen_open("effortless_building")`.
- `native_screen_render_post`: draws only when `screen.id == "mod"` and
  `native_mod_screen()` reports this mod as the owner.
- `native_screen_mouse_button_pressed_pre`: handles left clicks in the panel,
  queues a tool selection or close action, and cancels the press.
- `native_screen_mouse_button_released_pre`: cancels the matching captured
  left release so the press/release pair stays within the UI.
- `native_tick`: applies the queued setting change only on
  `native_window()`; the server's own tick cannot consume client UI state.
- `native_fill`, `native_text` and `native_text_width`: draw the panel
  with the game's own GUI atlas and text renderer.

The package metadata declares native hooks API 111 because the custom native
mod screen helper was added at that API level.

## Touch path in the supplied source

`crates/horizon-client/src/game.rs` routes `WindowEvent::Touch` through
`TouchPointer::event`. In `crates/horizon-client/src/touch.rs`, the pointer
maps movement to `CursorMoved` and press/release to left-button
`MouseInput`. This supports the native screen's mouse-button event path.
It is source-level evidence only: it does not replace a tap test on the
POCO F5 running Starling 0.8.0.

## Safety boundary

This milestone changes only the per-mod selected-tool setting. It does not
call `native_read_columns`, `native_edit_blocks`, or any local block setter.
No preview or world modification is claimed. Placement remains disabled until
the full server-authoritative adapter is implemented and tested.

## Validation state

The package workflow validates the archive contents and metadata. The current
CI does not compile this full mixin against the pinned Horizon tree. The
Starling launcher build and Android in-game tap/render test remain required.
