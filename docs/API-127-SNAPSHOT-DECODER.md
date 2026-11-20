# API 127 column snapshot decoder

The pure core now includes a strict decoder for the pinned
`NativeColumnBlocks` response.

## Source-derived layout

The pinned Horizon helper returns:
- `palette: Vec<String>`
- `bases: Vec<Option<i32>>`, one per requested column
- `blocks: Vec<u16>`, column-by-column, bottom-first
- `tints: Vec<u32>`, three tint values per column

For a request with `below` and `above`, each column contributes
`below + above` palette indices. The first sampled y coordinate is
`base - below`; subsequent entries increment y by one.

## Decoder behavior

`decode_column_snapshot`:
- Requires one base per input column.
- Rejects any unloaded column (None base), rather than interpreting it as air.
- Requires the exact expected block-buffer length.
- Rejects palette indices outside the returned palette.
- Enforces a caller-provided total block cap.
- Checks coordinate conversion for overflow.
- Produces absolute positions and palette indices, without mutating a world.

The helper source says unloaded columns are represented with no base and air
indices in the block buffer. The decoder deliberately rejects those columns
for transaction preflight: air padding is not proof that the requested world
positions were actually loaded.

Four unit tests cover bottom-first y mapping, unloaded columns, malformed
palette/buffer data, and the operation cap. Tests have not been executed in
this environment.

## Still required in the live adapter

The caller must:
1. Group requested positions by unique x/z columns.
2. Choose a bounded vertical range that fully covers each intended y.
3. Split calls to respect Horizon's 1,048,576-block read limit.
4. Compare decoded snapshot positions against the exact intended edit set.
5. Resolve palette strings to stable block-state identities and reject
   unsupported block entities/NBT before presenting confirmation.

This decoder is not itself a Horizon hook and does not read or write the game.
