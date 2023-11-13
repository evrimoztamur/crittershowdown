use serde::{Deserialize, Serialize};

use crate::{Lobby, LobbyError, LobbySettings};

/// A network message.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Everything's zappin'!
    Ok,
    // /// A single [`Turn`].
    // Move(Turn),
    // /// A list of [`Turn`]s for synchronising observers who may be multiple turns behind.
    // Moves(Vec<Turn>),
    /// An entire [`Lobby`] state for complete synchronisation.
    Lobby(Box<Lobby>),
    /// A [`LobbyError`].
    LobbyError(LobbyError),
}

/// An HTTP request made with a certain session ID.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRequest {
    /// The session ID for this request.
    pub session_id: String,
}

/// An HTTP request made with a session ID, containing a [`Message`] payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMessage {
    /// The session ID for this request.
    pub session_id: String,
    /// A [`Message`] payload.
    pub message: Message,
}

/// An HTTP request made with a session ID, containing a [`Message`] payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionNewLobby {
    /// A [`Message`] payload.
    pub lobby_settings: LobbySettings,
}
