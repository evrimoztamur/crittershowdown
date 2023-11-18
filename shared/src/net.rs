use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json_any_key::*;
use crate::{Lobby, LobbyError, LobbySettings, Turn};

/// A network message.
#[derive(Serialize, Deserialize)]
pub enum Message {
    /// Everything's zappin'!
    Ok,
    /// A single [`Turn`].
    Move(Turn),
    /// A list of [`Turn`]s for synchronising observers who may be multiple turns behind.
    TurnSync(Vec<Turn>),
    /// An entire [`Lobby`] state for complete synchronisation.
    Lobby(Box<Lobby>),
    /// List of lobbies
    Lobbies(#[serde(with = "any_key_map")] HashMap<u16, Lobby>),
    /// A [`LobbyError`].
    LobbyError(LobbyError),
}

/// An HTTP request made with a certain session ID.
#[derive(Serialize, Deserialize)]
pub struct SessionRequest {
    /// The session ID for this request.
    pub session_id: String,
}

/// An HTTP request made with a session ID, containing a [`Message`] payload.
#[derive(Serialize, Deserialize)]
pub struct SessionMessage {
    /// The session ID for this request.
    pub session_id: String,
    /// A [`Message`] payload.
    pub message: Message,
}

/// An HTTP request made with a session ID, containing a [`Message`] payload.
#[derive(Serialize, Deserialize)]
pub struct SessionNewLobby {
    /// The session ID for this request.
    pub session_id: String,
    /// A [`Message`] payload.
    pub lobby_settings: LobbySettings,
}
