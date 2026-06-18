//! The frontier and the ungoverned dark (#623): the cause-and-effect chain by
//! which the settled lands shed their restless. A young soul in a village worn
//! by hunger, feud, or want — with little to hold them — may take the road into
//! the open country between the new nations, beyond any town's reach.
//!
//! This module holds the frontier's own state. Slice 1 is only the seed: the
//! count of wanderers the settled lands have lost to the dark, fed by the
//! leave-for-the-road path in `migration`. Later slices turn enough gathered
//! wanderers into bands — living agents that roam, prey, and sometimes settle
//! an outlaw-hold that may, in time, become a town like any other.

use serde::{Deserialize, Serialize};

/// The ungoverned country beyond the settled lands, and who has gone into it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Frontier {
    /// Souls who have left the towns for the open country and not joined another
    /// settlement — drifters in the dark. The raw material of the bands to come.
    #[serde(default)]
    pub wanderers: u32,
}

impl Frontier {
    /// A soul takes the road into the ungoverned country.
    pub fn take_the_road(&mut self) {
        self.wanderers = self.wanderers.saturating_add(1);
    }
}
