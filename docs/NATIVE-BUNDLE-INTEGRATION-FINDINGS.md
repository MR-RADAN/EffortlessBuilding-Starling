# Native bundle integration findings at the Starling 0.8.0 Horizon pin

Source inspected at Horizon commit
`ac874422ed8f892505ff73d67ae486ecfee7a516`.

## Metadata shape

A native bundle metadata file is JSON with:
- `schema`
- `game`
- `id`
- `version`
- `mixins`

The in-tree Wrench example at this Horizon commit has schema 1 and records
its own Mixins commit in `mixins`. Its recorded Mixins revision differs from
this project's current pin, so its metadata is evidence for the field layout
only—not a compatible ready-made manifest. The final metadata must name the
exact Mixins revision accepted by the Starling 0.8.0 build service.

## Support-module pattern

The in-tree Wrench bundle declares support modules into
`crates/horizon-client/src/game.rs`, including a pure rules module and a
separate game adapter module. This confirms that the pinned native-mod system
uses same-crate support modules as an intended integration mechanism.

For this project, the clean-room geometry/planning module can remain
game-independent. A separate adapter module can own the Horizon-specific
types and hook calls. This split is preferable to mixing world API types into
the pure geometry code.

## Server authority warning

The Wrench adapter's own comments distinguish the local authoritative game
from a remote client connected to a server. That is consistent with this
project's requirement: world edits must only occur through the authoritative
server-side game context. A remote client must not simulate successful edits
locally or bypass server permissions.

## Screen and input status

The inspected bundle demonstrates mod list/settings integration, not proof
that the native screen's mouse hooks receive Android touch events. The
POCO F5 touch-input release blocker remains open.

## No executable manifest committed yet

The metadata's `mixins` field, native hook version, target module paths, and
helper bindings must all agree with the exact build service configuration.
The in-tree Wrench bundle uses a different Mixins hash than this project's pin;
copying its metadata unchanged would be incorrect. The next safe step is to
resolve Starling 0.8.0's build-service Mixins revision and package validator
contract before creating a bundle advertised as buildable.
