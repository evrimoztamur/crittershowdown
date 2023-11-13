#![forbid(missing_docs)]

//! The `shared` crate contains all the components which are used by both the client and the server, which includes the entire game logic too.

mod lobby;
mod logic;
mod net;
mod vecmap;

pub use lobby::*;
pub use logic::*;
pub use net::*;
pub use vecmap::*;
