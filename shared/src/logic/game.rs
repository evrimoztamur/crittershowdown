use serde::{Serialize, Deserialize};

use crate::{Team, Result};

/// Game structure.
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub struct Game {}

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
