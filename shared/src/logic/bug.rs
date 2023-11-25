use nalgebra::{vector, Vector2};
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
impl BugSort {
    fn max_health(&self) -> usize {
        match self {
            BugSort::WaterBeetle => 4,
            BugSort::FireBeetle => 4,
        }
    }
}

/// A bug
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub struct BugData {
    sort: BugSort,
    team: Team,
    impulse_intent: Vector2<f32>,
    health: usize,
}

impl BugData {
    /// Creates a new [`BugData`] entry.
    pub fn new(sort: BugSort, team: Team) -> BugData {
        BugData {
            sort,
            team,
            impulse_intent: Vector2::zeros(),
            health: sort.max_health(),
        }
    }
    /// Returns the [`BugSort`] for this [`Bug`].
    pub fn sort(&self) -> &BugSort {
        &self.sort
    }

    /// Returns the [`Team`] for this [`Bug`].
    pub fn team(&self) -> &Team {
        &self.team
    }

    /// Returns the intended impulse for this [`Bug`].
    pub fn impulse_intent(&self) -> &Vector2<f32> {
        &self.impulse_intent
    }

    /// TODO docs
    pub fn set_impulse_intent(&mut self, impulse_intent: Vector2<f32>) {
        let magnitude = impulse_intent.magnitude().min(4.0);

        self.impulse_intent = if impulse_intent.magnitude() > 0.05 {
            impulse_intent.normalize() * magnitude
        } else {
            vector![0.0, 0.0]
        };
    }

    /// helath
    pub fn health(&self) -> usize {
        self.health
    }

    /// set health
    pub fn add_health(&mut self, delta: isize) {
        self.health = (self.health as isize + delta).clamp(0, self.sort.max_health() as isize) as usize;
    }

    /// TODO docs
    pub fn reset_impulse_intent(&mut self) {
        self.impulse_intent = Vector2::zeros();
    }
}
