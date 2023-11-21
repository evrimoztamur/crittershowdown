use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{Game, Message, Team, Turn};

// #[cfg(feature = "server")]
// use crate::Turn;
// use crate::{Board, Game, Level, Mage, MageSort, Message, Team};

/// A identifier for a lobby, shared by the client and the server.
pub type LobbyID = u16;

/// Errors concerning the [`Lobby`].
#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyError(pub String);

impl<T> From<Result<T, LobbyError>> for Message {
    fn from(result: Result<T, LobbyError>) -> Self {
        match result {
            Ok(_) => Message::Ok,
            Err(err) => Message::LobbyError(err),
        }
    }
}

/// A player in a lobby, used in online lobbies only.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    /// The player's team.
    pub team: Team,
    /// Whether the player wants to rematch or not.
    pub rematch: bool,
    /// Last heartbeat.
    pub last_heartbeat: f64,
}

impl Player {
    fn new(team: Team, heartbeat: f64) -> Player {
        Player {
            team,
            rematch: false,
            last_heartbeat: heartbeat,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.team == other.team
    }
}

/// Settings for lobby
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LobbySettings {
    sort: LobbySort,
}

impl LobbySettings {
    /// Create a new instance of [`LobbySettings`].
    pub fn new(sort: LobbySort) -> LobbySettings {
        LobbySettings { sort }
    }

    /// Returns the [`LobbySort`].
    pub fn sort(&self) -> &LobbySort {
        &self.sort
    }

    /// Sets
    pub fn set_sort(&mut self, sort: LobbySort) {
        self.sort = sort;
    }
}

/// [`Lobby`] is a `struct` which contains all the information necessary for executing a game.
#[derive(Clone, Serialize, Deserialize)]
pub struct Lobby {
    /// The active [`Game`] of this lobby.
    #[serde(skip)]
    pub game: Game,
    players: HashMap<String, Player>,
    player_slots: VecDeque<Player>,
    /// Last heartbeat.
    pub first_heartbeat: f64,
    /// The [`Lobby`]s sort.
    pub settings: LobbySettings,
}

impl Lobby {
    /// Instantiates the [`Lobby`] `struct` with a given [`LobbySort`].
    pub fn new(settings: LobbySettings, first_heartbeat: f64) -> Lobby {
        // let mut rng = ChaCha8Rng::seed_from_u64(settings.seed);

        Lobby {
            game: Game::default(),
            players: HashMap::new(),
            player_slots: VecDeque::from([
                Player::new(Team::Red, 0.0),
                Player::new(Team::Blue, 0.0),
            ]),
            first_heartbeat,
            settings,
        }
    }

    /// Determines if all players slots are taken.
    pub fn all_ready(&self) -> bool {
        self.player_slots.is_empty()
    }

    #[cfg(feature = "server")]
    /// Includes a new session ID into the lobby, and assigns a player index to it.
    pub fn join_player(&mut self, session_id: String, timestamp: f64) -> Result<(), LobbyError> {
        if self.all_ready() {
            Err(LobbyError("cannot join an active game".to_string()))
        } else if self.players.contains_key(&session_id) {
            Err(LobbyError("already in lobby".to_string()))
        } else if let Some(mut player) = self.player_slots.pop_front() {
            player.last_heartbeat = timestamp;

            self.players.insert(session_id.clone(), player);

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
        use std::time::{SystemTime, UNIX_EPOCH};

        fn timestamp() -> f64 {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards");

            since_the_epoch.as_secs_f64()
        }

        if !self.all_ready() {
            Err(LobbyError("game not yet started".to_string()))
        } else {
            match self.players.get_mut(&session_id) {
                Some(player) => {
                    self.game.act_player(player, message);

                    player.last_heartbeat = timestamp();

                    Ok(())
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

    // /// Makes a fully-reset clone of this [`Lobby`].
    // pub fn remake(&mut self) {
    //     *self = Lobby::new(self.settings.clone());
    // }

    /// Determines if the game is finished.
    pub fn finished(&self) -> bool {
        self.game.result().is_some()
    }

    /// Determines if the game is local (`true`) or online.
    pub fn is_local(&self) -> bool {
        !matches!(self.settings.sort, LobbySort::Online(_))
    }

    /// Returns `true` for [`LobbySort::LocalAI`].
    pub fn has_ai(&self) -> bool {
        matches!(self.settings.sort, LobbySort::LocalAI)
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

    /// turns
    pub fn turns(&self) -> &Vec<Turn> {
        &self.game.turns()
    }

    /// Checks if any players are connected to this lobby
    pub fn any_connected(&self, timestamp: f64) -> bool {
        self.players
            .iter()
            .any(|(_, player)| timestamp - player.last_heartbeat < 15.0)
    }

    /// last bewat
    pub fn last_beat(&self) -> f64 {
        if let Some(turn) = self.game.last_turn() {
            turn.timestamp
        } else {
            self.first_heartbeat
        }
    }
}

/// Loadout methods.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
pub enum LobbySort {
    /// Choose the default order..
    #[default]
    Local,
    /// Versus AI.
    LocalAI,
    /// Online.
    Online(u16),
}
