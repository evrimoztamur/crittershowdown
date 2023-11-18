use std::collections::HashMap;

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// A turn
pub struct Turn {
    /// List of impulse intents
    #[serde(with = "any_key_map")]
    pub impulse_intents: HashMap<usize, Vector2<f32>>,
    /// time stamp
    pub timestamp: f64
}
