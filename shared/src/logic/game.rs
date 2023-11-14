use serde::{Deserialize, Serialize};

use crate::{Result, Team, Bug};

/// Game structure.
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Game {
    bugs: Vec<Bug>,
}

impl Game {
    /// Returns the [`Team`] that is to take their turn.
    pub fn turn_for(&self) -> Team {
        Team::Red
    }

    /// Returns the result of the [`Game`].
    pub fn result(&self) -> Option<Result> {
        None
    }
}
