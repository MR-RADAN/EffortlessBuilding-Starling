# Server-side world-edit adapter — Starling 0.9.0

Creator: MR  
Mod version: 0.1.0  
Target revision: `ff303502a4f7982551a0e0d3622d9268efe9e829`

## Supported entry point

The authoritative edit adapter belongs in the actual mod crate's root hook
implementation, not in a client `Game<'_>` mixin:

```rust,ignore
use mod_api::{self, Hooks, ServerHost};

#[derive(Default)]
struct EffortlessBuilding;

#[mod_api::horizon_mod]
impl Hooks for EffortlessBuilding {
    fn server_tick(
        &self,
        host: &mut dyn ServerHost,
        event: &mut mod_api::ServerTickEvent,
    ) {
        // Process only a bounded, explicitly confirmed server-side request.
    }
}
```

The pinned hook catalog declares `fn server_tick()` as a server-side listen
hook. The generated API dispatches it in two phases: `ServerTickStart` before
any level ticks and `ServerTickEnd` after the last level tick. A paused pass
fires no event; a frozen world reports `world_running == false` except during
a requested step. `host.server_has_time()` reports whether optional work still
fits before the next tick. Do not assume the current world/level is the target
dimension: the API documentation explicitly says which level the server host
holds is not defined. The server hook is the authority boundary; the existing
client mixin remains responsible only for selection input, UI and drawing.

## Public host calls

The pinned `ServerHost` world-edit surface declares:

- `surface_heights(dimension, columns, heightmap)`
- `read_columns(dimension, columns, heightmap, range)`
- `edit_blocks(dimension, edit)`

Limits are 65,536 columns for a height query and 1,048,576 block positions
per column read. Unloaded columns produce `None` in the height preflight,
but read results represent unloaded columns as air. Every target column must
therefore pass the height preflight before a snapshot is accepted.

`edit_blocks` takes a `mod_api::BlockEdit`, whose palette contains complete
block states and whose runs refer to palette entries and optional filters.
It may omit runs in unloaded chunks or outside the column. The adapter must
preflight the exact target bounds and treat a returned count different from
the intended count as a failed/incomplete commit; it must not record history
for that operation.

## Required request lifecycle

1. Client input creates a selection request only. It cannot call world writes.
2. Server hook validates the request identity, dimension, world/session epoch,
   operation cap (16,384), and explicit confirmation token.
3. Resolve unique target columns; preflight all of them with
   `surface_heights`. Abort on any unloaded column.
4. Read the full exact target Y span in bounded calls. Decode the column-major,
   bottom-first palette indices while preserving complete palette states.
5. Construct `BlockEdit` runs and per-position expected-state predicates from
   that fresh snapshot. Validate palette/filter indexes before the write.
6. Call `edit_blocks` once for the confirmed request. Only a complete
   expected change count may be recorded in history.
7. Undo/redo repeat the preflight and fresh snapshot, and refuse if any current
   state differs from the expected state recorded by the transaction.

## Important limitation

The API offers separate read and edit calls, not a multi-call world lock or
rollback transaction. This is optimistic conflict checking; an external world
change may occur between snapshot and edit. Never describe it as strict
atomicity, and never claim rollback after an uncertain/partial engine result.

## Current implementation state

The client-side mixin bundle does not yet contain this server hook crate or
connect it to the shared selection/request state. No live block mutation,
inventory consumption, unlimited-block mode, or live undo/redo is enabled.
This document records the verified integration boundary; it is not a claim
that the adapter has been implemented or tested in-game.


## Critical packaging distinction

The currently installed `.hnmod` is a legacy `horizon-mixins` bundle. Its
`Game<'_>` injections do not receive `&mut dyn ServerHost`; adding a method
with a ServerHost-like signature inside `native.rs` cannot make that API
available. The `ServerHost` route above describes a separate Horizon code-mod
crate using `horizon-mod-api`, not a callable helper in this mixin bundle.

Starling's Wrench native mod demonstrates the alternative supported path for
the current bundle: mutate only the acting single-player/LAN server's
`Game` world through its existing `with_level` / level block APIs, and refuse
remote clients. This is likely the smaller integration path for this project,
but it still needs a carefully preflighted transaction adapter and verified
rollback/partial-failure handling before it can be enabled.
