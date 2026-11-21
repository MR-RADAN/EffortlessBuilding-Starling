# Starling 0.9.0 Android device validation

This checklist is intentionally not marked passed until tested on a real device.
The current target is Starling 0.9.0, Android ARM64, and the POCO F5 target
configuration recorded in the project notes.

## Import and activation

- Confirm the launcher is running Starling 0.9.0 and Android ARM64.
- Import the generated `effortless_building-0.1.0.hnmod` source package using
  Starling's supported native build/import flow.
- Confirm the native library is built for the exact installed APK and its
  SHA-256 is accepted by the launcher.
- Enable Effortless Building in Options > Mods and open its settings panel.

## Selection and preview

- Select Floor, close the panel, equip a stick, and right-click a visible block.
  The screen should report point A when reopened.
- Right-click a second visible block. The screen should report A and B plus
  the target-position count for the selected tool.
- Confirm the world shows a read-only wireframe and no more than 64 distributed
  sample crosses. Markers should span the full selected bounds, not cluster
  around the player or one corner.
- Select Wall, Pillar and Cube in turn and verify the preview bounds and target
  counts match their documented two-corner geometry.
- Right-click a third block and verify it replaces A and clears B.
- Hold a non-stick item and right-click; normal item use must continue.
- Verify no block state changes during any selection or preview step.

## Safety and compatibility

- In a remote multiplayer client, verify the selection hook does not consume
  the click and no world edit is attempted.
- Verify the screen does not pause the single-player world.
- Verify touch input remains compatible with Starling, MCPE and GoyControl;
  no existing touch layout should be rewritten.
- Test small and large selections, including a selection above 16,384 target
  positions. The latter must show the cap warning and must never offer a commit.
- Rotate the camera, move away from the selection and return; preview markers
  should remain attached to the selected world coordinates and remain depth
  tested behind opaque terrain.

## Not part of this preview milestone

- No block placement or removal.
- No survival inventory consumption.
- No Unlimited Blocks mode.
- No block-entity/NBT preservation.
- No live undo/redo.
- No multiplayer client-side edits.

Record the device model, Android version, Starling APK version, build artifact
SHA-256, and each pass/fail result before declaring runtime validation complete.
