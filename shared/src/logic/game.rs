use std::collections::HashMap;

use nalgebra::{vector, Point2, Vector2};
use rapier2d::dynamics::{RigidBody, RigidBodyHandle};
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;

use crate::{BugData, Message, Physics, Player, Result, Team};

/// Game structure.
#[derive(Clone)]
pub struct Game {
    physics: Physics,
    bugs: HashMap<usize, BugData>,
    bug_handles: HashMap<usize, RigidBodyHandle>,
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Game {
            physics: Physics::default(),
            bugs: HashMap::new(),
            bug_handles: HashMap::new(),
        };

        for i in 0..16 {
            game.insert_bug(
                vector![0.0 + (i as f32).cos() * 2.0, 0.0 + (i as f32).sin() * 2.0],
                BugData::new(crate::BugSort::WaterBeetle, Team::Blue),
            );
        }

        game
    }
}
impl Game {
    /// Returns the result of the [`Game`].
    pub fn result(&self) -> Option<Result> {
        None
    }

    /// Advances the [`Game`] simulation by one tick.
    pub fn tick(&mut self) {
        // let random = (Math::random() * self.physics.collider_set.len() as f64).floor() as usize;

        // for (i, (_, rb)) in self.physics.rigid_body_set.iter_mut().enumerate() {
        //     if i == random {
        //         rb.apply_impulse(
        //             (vector![Math::random() as f32 - 0.5, Math::random() as f32 - 0.5]).scale(2.0),
        //             true,
        //         );
        //     }
        // }

        self.physics.tick();
    }

    /// Find the [`Bug`] that's the closest to the given [`Point2`].
    pub fn intersecting_bug(&self, point: Point2<f32>) -> Option<(usize, &RigidBody, &BugData)> {
        if let Some((collider_handle, _)) = self.physics.intersecting_collider(point) {
            if let Some(collider) = self.physics.collider_set.get(collider_handle) {
                if let Some(collider_parent_handle) = collider.parent() {
                    if let Some(rigid_body) =
                        self.physics.rigid_body_set.get(collider_parent_handle)
                    {
                        if let Some(data) = self.bugs.get(&(rigid_body.user_data as usize)) {
                            Some(((rigid_body.user_data as usize), rigid_body, data))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Find the [`Bug`] that's the closest to the given [`Point2`].
    pub fn intersecting_bug_mut(
        &mut self,
        point: Point2<f32>,
    ) -> Option<(usize, &RigidBody, &mut BugData)> {
        if let Some((collider_handle, _)) = self.physics.intersecting_collider(point) {
            if let Some(collider) = self.physics.collider_set.get(collider_handle) {
                if let Some(collider_parent_handle) = collider.parent() {
                    if let Some(rigid_body) =
                        self.physics.rigid_body_set.get(collider_parent_handle)
                    {
                        if let Some(data) = self.bugs.get_mut(&(rigid_body.user_data as usize)) {
                            Some(((rigid_body.user_data as usize), rigid_body, data))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns an iterator over all active [`Bugs`].
    pub fn iter_bugdata(&self) -> impl Iterator<Item = &BugData> {
        self.bugs.values()
    }

    /// Returns an iterator over all active [`Bugs`].
    pub fn iter_bugs(&self) -> impl Iterator<Item = (&RigidBody, &BugData)> {
        self.physics
            .rigid_body_set
            .iter()
            .filter_map(|(rigid_body_handle, rigid_body)| {
                self.bugs
                    .get(&(rigid_body.user_data as usize))
                    .and_then(|data| Some((rigid_body, data)))
            })
    }

    /// Returns an iterator over all active [`Bugs`].
    pub fn iter_bugmuts(&mut self) -> impl Iterator<Item = (&mut RigidBody, &BugData)> {
        self.physics
            .rigid_body_set
            .iter_mut()
            .filter_map(|(rigid_body_handle, rigid_body)| {
                self.bugs
                    .get(&(rigid_body.user_data as usize))
                    .and_then(|data| Some((rigid_body, data)))
            })
    }

    /// Inserts a new [`Bug`].
    pub fn insert_bug(
        &mut self,
        translation: Vector2<f32>,
        bug_data: BugData,
    ) -> (usize, RigidBodyHandle) {
        let bug_index = self.bugs.len();
        let rigid_body_handle = self.physics.insert_bug(translation, bug_index);

        self.bugs.insert(bug_index, bug_data);
        self.bug_handles.insert(bug_index, rigid_body_handle);

        (bug_index, rigid_body_handle)
    }

    /// Shoots all [`Bug`]s forward based on their impulses.
    pub fn execute_turn(&mut self) {
        for (rigid_body, data) in self.iter_bugmuts() {
            rigid_body.apply_impulse(*data.impulse_intent() * 2.0, true)
        }

        for bug_data in self.bugs.values_mut() {
            bug_data.reset_impulse_intent();
        }
    }

    /// TODO docs
    pub fn get_bug(&self, bug_index: usize) -> Option<(&RigidBody, &BugData)> {
        if let (Some(bug_data), Some(bug_handle)) =
            (self.bugs.get(&bug_index), self.bug_handles.get(&bug_index))
        {
            if let Some(rigid_body) = self.physics.rigid_body_set.get(*bug_handle) {
                Some((rigid_body, bug_data))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// TODO docs
    pub fn get_bug_mut(&mut self, bug_index: usize) -> Option<(&mut RigidBody, &mut BugData)> {
        if let (Some(bug_data), Some(bug_handle)) = (
            self.bugs.get_mut(&bug_index),
            self.bug_handles.get_mut(&bug_index),
        ) {
            if let Some(rigid_body) = self.physics.rigid_body_set.get_mut(*bug_handle) {
                Some((rigid_body, bug_data))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// TODO docs
    pub fn get_bug_from_handle(
        &self,
        bug_handle: RigidBodyHandle,
    ) -> Option<(&RigidBody, &BugData)> {
        if let Some(rigid_body) = self.physics.rigid_body_set.get(bug_handle) {
            let bug_data = self.bugs.get(&(rigid_body.user_data as usize));

            if let Some(bug_data) = bug_data {
                Some((rigid_body, bug_data))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// TODO docs
    pub fn get_bug_from_handle_mut(
        &mut self,
        bug_handle: RigidBodyHandle,
    ) -> Option<(&mut RigidBody, &mut BugData)> {
        if let Some(rigid_body) = self.physics.rigid_body_set.get_mut(bug_handle) {
            let bug_data = self.bugs.get_mut(&(rigid_body.user_data as usize));

            if let Some(bug_data) = bug_data {
                Some((rigid_body, bug_data))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Processes message for player
    pub fn act_player(&mut self, player: &Player, message: Message) {
        match message {
            Message::Ok => (),
            Message::Move(turn) => {
                for (bug_index, impulse_intent) in turn.impulse_intents {
                    if let Some(bug_data) = self.bugs.get_mut(&bug_index) {
                        if bug_data.team() == &player.team {
                            bug_data.set_impulse_intent(impulse_intent);
                        }
                    }
                }
            }
            Message::Moves(_) => (),
            Message::Lobby(_) => (),
            Message::Lobbies(_) => (),
            Message::LobbyError(_) => (),
        }
    }
}
