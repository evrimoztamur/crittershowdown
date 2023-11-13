use std::collections::{BTreeMap, HashMap, VecDeque};

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
// use crate::Turn;
// use crate::{Board, Game, Level, Mage, MageSort, Message, Team};

/// A identifier for a lobby, shared by the client and the server.
pub type LobbyID = u16;

/// Errors concerning the [`Lobby`].
#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyError(pub String);

// impl<T> From<Result<T, LobbyError>> for Message {
//     fn from(result: Result<T, LobbyError>) -> Self {
//         match result {
//             Ok(_) => Message::Ok,
//             Err(err) => Message::LobbyError(err),
//         }
//     }
// }

/// A player in a lobby, used in online lobbies only.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    /// The player's team.
    pub team: Team,
    /// Whether the player wants to rematch or not.
    pub rematch: bool,
}

impl Player {
    fn new(team: Team) -> Player {
        Player {
            team,
            rematch: false,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.team == other.team
    }
}

/// [`Lobby`] is a `struct` which contains all the information necessary for executing a game.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lobby {
    /// The active [`Game`] of this lobby.
    pub game: Game,
    players: HashMap<String, Player>,
    player_slots: VecDeque<Player>,
    ticks: usize,
    /// The [`Lobby`]s sort.
    pub settings: LobbySettings,
}

impl Lobby {
    /// Instantiates the [`Lobby`] `struct` with a given [`LobbySort`].
    pub fn new(settings: LobbySettings) -> Lobby {
        let mut rng = ChaCha8Rng::seed_from_u64(settings.seed);

        Lobby {
            game: Game::new(&settings.level(&mut rng), settings.can_stalemate)
                .expect("game should be instantiable with default values"),
            players: HashMap::new(),
            player_slots: VecDeque::from([Player::new(Team::Red), Player::new(Team::Blue)]),
            ticks: 0,
            settings,
        }
    }

    /// Number of ticks since the lobby's creation.
    /// Used to synchronise lobby-related events.
    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    /// Determines if all players slots are taken.
    pub fn all_ready(&self) -> bool {
        self.player_slots.is_empty()
    }

    #[cfg(feature = "server")]
    /// Includes a new session ID into the lobby, and assigns a player index to it.
    pub fn join_player(&mut self, session_id: String) -> Result<(), LobbyError> {
        if self.all_ready() {
            Err(LobbyError("cannot join an active game".to_string()))
        } else if self.players.contains_key(&session_id) {
            Err(LobbyError("already in lobby".to_string()))
        } else if let Some(player) = self.player_slots.pop_front() {
            self.players.insert(session_id.clone(), player);

            self.tick();

            Ok(())
        } else {
            Err(LobbyError("no available slots in lobby".to_string()))
        }
    }

    // #[cfg(feature = "server")]
    // pub fn leave_player(&mut self, session_id: String) -> Result<String, LobbyError> {
    //     if self.state == LobbyState::Finished {
    //         Err(LobbyError("cannot leave a finished game".to_string()))
    //     } else {
    //         match self.players.remove(&session_id) {
    //             Some(player) => {
    //                 self.player_slots.push_back(player.index);

    //                 self.players.remove(&session_id);

    //                 self.tick();

    //                 Ok(session_id)
    //             }
    //             None => Err(LobbyError("player not in lobby".to_string())),
    //         }
    //     }
    // }

    #[cfg(feature = "server")]
    /// Executes a certain [`Message`] for the player.
    pub fn act_player(&mut self, session_id: String, message: Message) -> Result<(), LobbyError> {
        if !self.all_ready() {
            Err(LobbyError("game not yet started".to_string()))
        } else {
            match self.players.get(&session_id) {
                Some(player) => {
                    if self.game.turn_for() == player.team {
                        if let Message::Move(Turn(from, to)) = message {
                            self.game.take_move(from, to);
                        }

                        Ok(())
                    } else {
                        Err(LobbyError("not your turn".to_string()))
                    }
                }
                None => Err(LobbyError("player not in lobby".to_string())),
            }
        }
    }

    #[cfg(feature = "server")]
    /// Requests a rematch for the active game.
    pub fn request_rematch(&mut self, session_id: String) -> Result<bool, LobbyError> {
        if !self.all_ready() {
            Err(LobbyError("game not yet started".to_string()))
        } else {
            match self.players.get_mut(&session_id) {
                Some(player) => {
                    player.rematch = true;

                    Ok(self
                        .players
                        .values()
                        .fold(true, |acc, player| acc & player.rematch))
                }
                None => Err(LobbyError("player not in lobby".to_string())),
            }
        }
        // else if !self.finished() {
        //     Err(LobbyError("game not yet finished".to_string()))
        // }
    }

    /// Makes a fully-reset clone of this [`Lobby`].
    pub fn remake(&mut self) {
        *self = Lobby::new(self.settings.clone());
    }

    /// Determines if the game is finished.
    pub fn finished(&self) -> bool {
        self.game.result().is_some()
    }

    /// Determines if the game is local (`true`) or online.
    pub fn is_local(&self) -> bool {
        !matches!(self.settings.lobby_sort, LobbySort::Online(_))
    }

    /// Returns `true` for [`LobbySort::LocalAI`].
    pub fn has_ai(&self) -> bool {
        matches!(self.settings.lobby_sort, LobbySort::LocalAI)
    }

    /// Determines if the given session ID is the one taking its turn.
    pub fn is_active_player(&self, session_id: Option<&String>) -> bool {
        if self.is_local() {
            !(self.has_ai() && self.game.turn_for() == Team::Blue)
        } else if !self.all_ready() {
            false
        } else {
            match session_id {
                Some(session_id) => match self.players.get(session_id) {
                    Some(player) => self.game.turn_for() == player.team,
                    None => false,
                },
                None => false,
            }
        }
    }

    /// Detemines whether or not the given session ID is in this lobby.
    pub fn has_session_id(&self, session_id: Option<&String>) -> bool {
        match session_id {
            Some(session_id) => self.players.contains_key(session_id),
            None => false,
        }
    }

    /// Returns the players.
    pub fn players(&self) -> &HashMap<String, Player> {
        &self.players
    }

    /// Rewinds the [`Game`] by `delta` turns.
    pub fn rewind(&mut self, delta: usize) {
        let rewinded_game = self.game.rewind(delta);

        self.game = rewinded_game;

        // This is a state-modfying action and in turn must be communicated with the connected clients.
        // Rewinding is currently only available in local lobbies, but take-backs may be implemented.
        self.tick();
    }
}
