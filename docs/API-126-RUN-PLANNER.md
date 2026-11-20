# Horizon API 126 run planner

The pure Rust core now has `native_runs`, a bounded converter from absolute
`BlockPos` values to the vertical-run representation used by the pinned
Horizon API 126 `NativeBlockRun`.

## Mapping

The planner mirrors the pinned source fields:
- `x: i32`, `z: i32`, `y: i32`
- `height: u32`
- `surface: Option<NativeHeightmapPlan>`
- `block: u32`
- `only: Option<u32>`

For the first placement path it emits absolute-y runs only:
`surface = None`, `only = None`, and one supplied palette index in
`block`. Heightmap-relative placement and filters are deliberately excluded
until separately implemented and preflighted.

## Safety properties

- Sorts deterministically by x, z, then y.
- Deduplicates repeated positions.
- Coalesces only directly adjacent y coordinates in the same x/z column.
- Never bridges gaps or merges separate columns.
- Uses checked height increments.
- Rejects input positions above the configured cap and output runs above the
  configured cap.
- Uses i64 comparison for adjacency so i32 coordinate boundaries cannot wrap.

The function only prepares a request. It does not read the world, verify chunk
loading, check block permissions, call the server edit helper, or mutate a
world. The game adapter must complete those steps before any commit.

Unit tests were added for coalescing, gaps, column boundaries, duplicates,
limits, and signed y boundaries. They have not been run in this environment:
Rust/Cargo and the pinned Starling build service are not available here.
