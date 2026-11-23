# Effortless Building diagnostics

The native bundle writes local diagnostic events to the game's shared config
directory, using Starling/Horizon's `Game::native_config_dir()` API.

## File location

`config/effortless_building/diagnostics.jsonl`

The previous rotated file is:

`config/effortless_building/diagnostics.previous.jsonl`

Each line is an independent JSON object with a Unix-millisecond timestamp,
mod id/version, event name and bounded detail string. Events currently include
selection-point capture, tool selection, the read-only preflight result and
builder close.

## Retention and privacy

- The active file is rotated after it reaches 256 KiB.
- Only the current and one previous log are retained.
- Detail strings are capped at 2,048 characters per event.
- Logs stay on the device. The mod does not upload or share them.
- Selection coordinates may appear in the log because they are useful when
  diagnosing point-selection and reach problems. Review the file before sharing.

## Sending a report

After reproducing a problem, close Starling normally and retrieve
`diagnostics.jsonl` (and, if present, `diagnostics.previous.jsonl`) from the
game's config folder. Send both files together with the Starling version,
Android version, and a short description of the steps that caused the problem.

This mod log supplements Starling's own log and crash-report facilities; it
does not replace Android logcat, native crash traces, or Starling's diagnostic
ZIP.
