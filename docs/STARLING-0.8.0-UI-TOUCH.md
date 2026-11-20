# Starling 0.8.0 UI and touch-input findings

Target: Starling v0.8.0 / Horizon commit
`ac874422ed8f892505ff73d67ae486ecfee7a516`.

## Verified native UI hooks

At this exact source revision, `native_hooks.rs` exposes the following mod-dispatch hooks:

- `native_mods(&mut self, mods: &mut Vec<NativeMod>)`: add the mod to Options > Mods.
- `native_key(&mut self, key: &str, pressed: bool)`: vanilla-named key/button events; repeats omitted. The hook runs on screens too; check `native_playing` before gameplay-only actions.
- `native_mod_screen_open(&mut self, mod_id: &str) -> bool`: opens a dedicated mod screen, but refuses while title/loading/dialog/another screen gates apply. It clears gameplay input and releases capture.
- `native_mod_screen_close(&mut self, mod_id: &str)` and `native_mod_screen(&self) -> Option<&str>`.
- `native_screen_render_post(&self, batch: &mut HudBatch, screen: &NativeScreenView<'_>)`: draw above an open screen.
- `native_screen_mouse_button_pressed_pre` and `native_screen_mouse_button_released_pre`: inspect/cancel real mouse-button events; the hook explicitly documents that gamepad clicks and keyboard-focus clicks do not arrive here.
- `native_screen_mouse_dragged_pre` and screen wheel/key/character hooks are available.
- `native_setting` / `native_set_setting` persist per-mod settings.

These are native-mod dispatch hooks, not direct public Rust API calls from arbitrary crates. A bundle must declare injections into the exact hook functions and provide the matching helper signatures. The exact hook catalog is also represented in `native-hooks.json`; use the API level required by the hooks actually consumed.

## UI architecture decision

Use the game's own custom native mod screen, not an overlay permanently layered over gameplay:
1. Register the mod through `native_mods`.
2. Open it through `native_mod_screen_open` from a configured native key/custom touch button.
3. Render a compact shape/action panel from `native_screen_render_post`.
4. Route touch/mouse clicks through the screen mouse hooks and handle Escape/close safely.
5. Keep world selection and edit transactions separate from UI state.

Starling's release notes also document configurable on-screen Custom Buttons that can send a native mod key. This provides a practical mobile entry point without drawing over movement/jump controls. However, the exact mapping and touch behavior still require an in-game test on Android.

## Important constraints

- Mouse hooks explicitly do not receive gamepad clicks or keyboard-focus clicks.
- The native screen open helper blocks when another screen/dialog is active and while loading.
- A successful Rust check or bundle validation is not proof that touch routing works in-game.
- Keep all world mutation on the authoritative server side. Client screen hooks should only collect intent and display preview; they must not mutate the local client world.
- Do not claim a ghost preview exists until a supported render path is implemented and tested. A coordinate/count preview is a safe first milestone.

## Current status

Research only. This document does not claim that a native bundle, Android build, or in-game test has been completed.
