use serde::{Deserialize, Serialize};

/// An `enum` for the teams. Currently there are only two teams, red and blue.
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone, Default)]
pub enum Team {
    /// Red team.
    #[default]
    Red,
    /// Blue team.
    Blue,
}

impl Team {
    /// Returns the team for a given mage index.
    pub fn from_index(index: usize) -> Team {
        match index % 2 {
            0 => Team::Red,
            _ => Team::Blue,
        }
    }

    /// Returns the opposing team.
    pub fn enemy(&self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

/// An `enum` for the game results.
#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Result {
    /// Win for a certain [`Team`].
    Win(Team),
    /// Tie.
    Tie,
}
