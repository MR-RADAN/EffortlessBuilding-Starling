# World-edit adapter contract

This is the implementation contract for the first safe game adapter. It is
derived from the pinned Horizon 0.8.0 helper source, not from upstream mod code.

## Supported mutation path

Use only the server-side `Game::native_edit_blocks` helper on the authoritative
level currently being ticked. Do not call a client-side world mutation path and
do not attempt to edit another dimension. Treat an error as a full abort.

The helper accepts a `NativeBlockEdit` with a block-state palette, optional
command-style filters, and vertical runs per `(x,z)` column. A run specifies
its start `y`, height, palette index, and optional heightmap-relative origin.
For arbitrary shape output, sort positions by `(x,z,y)` and coalesce only
strictly contiguous vertical positions with the same target state and filter.
Never bridge gaps or merge across columns.

## Preflight and snapshot

Before confirmation, the adapter must:

1. Resolve the target dimension and verify that the authoritative level is
   currently open for editing.
2. Read every affected column in bounded groups using the pinned column-read
   API. Split requests below the API's documented maximum.
3. Build a complete per-position before-state snapshot for the exact proposed
   edit. If any affected position is unreadable, unloaded, outside the
   returned range, or cannot be mapped unambiguously from the returned
   palette/base/block arrays, abort without editing.
4. Classify block states. The first release must reject block entities and
   states whose restoration semantics are not proven. Do not promise NBT
   preservation: the bulk edit helper explicitly behaves like `fill strict`
   for block entities.
5. Enforce the operation and memory caps before showing the final confirmation.

## Commit, undo and redo

Only after an explicit user confirmation may the adapter call the bulk-edit
helper. Store the exact before/after state for each accepted position, and
persist transaction records only after the helper reports success. A failed
or partial result must not be represented as a complete transaction.

Before undo or redo, read back every affected position and compare it with the
expected current state. If any position differs, reject the whole transaction
rather than overwriting later player or server changes. Reverse/forward edits
must pass through the same authoritative server helper.

Because the helper returns a changed-block count and can skip unloaded chunks
or out-of-column portions, require the reported count to equal the intended
number of changed positions. Otherwise mark the operation incomplete, do not
offer it as undoable, and report the discrepancy. The first implementation
must avoid heightmap-relative runs unless their exact resulting positions
have been validated against the preflight snapshot.

## Inventory and Unlimited Blocks

Do not infer inventory consumption from a successful world edit. Inventory
accounting requires a separate verified authoritative API and must be
implemented before survival-mode placement is advertised. Unlimited Blocks
must remain disabled until server-side permission and game-mode semantics are
verified. It must never bypass a multiplayer server's authority.

## Explicit non-goals for first adapter

- No edits to unloaded chunks.
- No block-entity/NBT restoration promise.
- No client-only or remote multiplayer world mutation.
- No best-effort partial transaction presented as successful.
- No undo that overwrites intervening world changes.

This contract does not claim that an adapter or package has been implemented
or compiled.