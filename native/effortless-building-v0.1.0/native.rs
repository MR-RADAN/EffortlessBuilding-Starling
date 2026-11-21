// In-game point selection for Effortless Building.
// This hook only records the block the player is already aiming at; it never
// changes the world. The authoritative world-edit adapter is a separate gate.
use super::{Game, GuestUse};
use horizon_client::first_person::Hand;

impl Game<'_> {
    /// Capture a main-hand block use as one corner of the build selection.
    /// Guests connected to another server are deliberately not intercepted.
    pub(super) fn effortless_building_use(&mut self) -> Option<Option<Hand>> {
        use crate::game::effortless_building as eb;

        if self.remote.is_some()
            || self.player.is_none()
            || self.spectator()
            || !self.native_on(eb::ID)
        {
            return None;
        }
        if self.lan_acting.is_none() && !self.native_window() {
            return None;
        }
        match self.guest_use {
            Some(GuestUse::OnBlock(Hand::Main)) => {}
            None if !self.entity_targeted() => {}
            _ => return None,
        }

        let hit = self.pick()?;
        let pos = hit.pos;
        let recorded = self.with_player_actor(|_game| {
            eb::record_selection_point(pos);
            true
        });
        if !recorded {
            return None;
        }
        self.swing_arm();
        Some(Some(Hand::Main))
    }
}
