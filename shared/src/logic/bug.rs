use rapier2d::dynamics::RigidBodyHandle;
use serde::{Deserialize, Serialize};

use crate::Team;

/// Sort of a bug
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub enum BugSort {
    /// A water beetle
    #[default]
    WaterBeetle,
    /// A fire beetle
    FireBeetle,
}

/// A bug
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Bug {
    sort: BugSort,
    team: Team,
    rigid_body_handle: RigidBodyHandle,
}

impl Bug {
    /// Returns the [`BugSort`] for this [`Bug`].
    pub fn sort(&self) -> &BugSort {
        &self.sort
    }

    /// Returns the [`Team`] for this [`Bug`].
    pub fn team(&self) -> &Team {
        &self.team
    }
}
