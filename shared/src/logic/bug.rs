use rapier2d::dynamics::RigidBody;
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
pub struct BugData {
    sort: BugSort,
    team: Team,
}

impl BugData {
    /// Creates a new [`BugData`] entry.
    pub fn new(sort: BugSort, team: Team) -> BugData {
        BugData { sort, team }
    }
    /// Returns the [`BugSort`] for this [`Bug`].
    pub fn sort(&self) -> &BugSort {
        &self.sort
    }

    /// Returns the [`Team`] for this [`Bug`].
    pub fn team(&self) -> &Team {
        &self.team
    }
}

/// A [`Bug`].
pub struct Bug<'a> {
    /// [`RigidBody`] of the [`Bug`].
    pub rigid_body: &'a RigidBody,
    /// [`BugData`] for the [`Bug`].
    pub data: &'a BugData,
}
