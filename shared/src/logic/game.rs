use std::collections::{HashMap, VecDeque};

use nalgebra::{vector, Point2, Vector2};
use rapier2d::dynamics::{RigidBody, RigidBodyHandle};
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;

use crate::{BugData, Message, Physics, Player, Result, Team, Turn};

/// Game structure.
#[derive(Clone)]
pub struct Game {
    physics: Physics,
    bugs: HashMap<usize, BugData>,
    bug_handles: HashMap<usize, RigidBodyHandle>,
    ticks: u64,
    turns: Vec<Turn>,
    queued_turns: VecDeque<Turn>,
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Game {
            physics: Physics::default(),
            bugs: HashMap::new(),
            bug_handles: HashMap::new(),
            turns: Vec::new(),
            queued_turns: VecDeque::new(),
            ticks: 0,
        };

        let team_size = 6;
        let num_bugs = team_size * 2;

        for i in 0..num_bugs {
            let offset = i % team_size;
            let arc_size = 0.3;
            let team_arc = arc_size * (team_size - 1) as f32;
            let arc_offset = team_arc / 2.0;
            let team_offset = if i < team_size {
                -arc_offset
            } else {
                std::f32::consts::PI - arc_offset
            };
            let net_offset = team_offset + arc_size * offset as f32;

            let team = if i < team_size { Team::Red } else { Team::Blue };

            game.insert_bug(
                vector![
                    0.0 + (net_offset).cos() * 4.0,
                    0.0 + (net_offset).sin() * 4.0
                ],
                BugData::new(crate::BugSort::WaterBeetle, team),
            );
        }

        game
    }
}
impl Game {
    /// Returns a list of [`Turn`]s skipping the first `since` turns.
    pub fn turns_since(&self, since: usize) -> Vec<&Turn> {
        self.turns.iter().skip(since).collect()
    }

    /// Returns the latest [`Turn`].
    pub fn last_turn(&self) -> Option<&Turn> {
        self.turns.last()
    }

    /// hypothetical turn
    pub fn aggregate_turn(&self) -> Turn {
        Turn {
            impulse_intents: HashMap::from_iter(
                self.bugs.iter().map(|(i, bug)| (*i, *bug.impulse_intent())),
            ),
            timestamp: 0.0,
            index: self.turns_count(),
        }
    }

    /// Returns the result of the [`Game`].
    pub fn result(&self) -> Option<Result> {
        None
    }

    /// num ticks
    ///
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Advances the [`Game`] simulation by one tick.
    pub fn tick(&mut self) {
        if self.ticks % (7 * 60) == 0 {
            // At each 7 second interval, check for queued turns (which are sent from the server
            if let Some(queued_turn) = self.queued_turns.pop_front() {
                if self.execute_turn(&queued_turn) {
                    self.subtick();
                }
            }
            // Do not act until available
        } else {
            self.subtick();
        }

        // Tick until we reach the next target
        if !self.queued_turns.is_empty() {
            self.tick();
        }
    }

    // /// target tick
    // pub fn target_tick(&self) -> u64 {
    //     ((self.all_turns_count() as f64) * 7.0 * 60.0).max(0.0) as u64
    // }

    /// force a subtick
    ///
    pub fn subtick(&mut self) {
        self.physics.tick();
        self.ticks += 1;
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

    /// records turns
    pub fn queue_turns(&mut self, turns: Vec<Turn>) {
        self.queued_turns.append(&mut VecDeque::from(turns));
    }

    /// Shoots all [`Bug`]s forward based on their impulses.
    pub fn execute_turn(&mut self, turn: &Turn) -> bool {
        let pass = if let Some(last_turn) = self.last_turn() {
            turn.index > last_turn.index
        } else {
            true
        };

        if pass {
            for (i, bug_data) in &mut self.bugs {
                if let Some(impulse_intent) = turn.impulse_intents.get(i) {
                    bug_data.set_impulse_intent(impulse_intent.clone());
                }
            }

            for (rigid_body, data) in self.iter_bugmuts() {
                rigid_body.apply_impulse(*data.impulse_intent() * 2.0, true)
            }

            self.reset_impulses();

            self.turns.push(turn.clone());
        }

        pass
    }

    /// reset impulses
    fn reset_impulses(&mut self) {
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
            Message::TurnSync(_) => (),
            Message::Lobby(_) => (),
            Message::Lobbies(_) => (),
            Message::LobbyError(_) => (),
        }
    }

    /// num turns
    pub fn turns(&self) -> &Vec<Turn> {
        &self.turns
    }

    /// num turns
    pub fn turns_count(&self) -> usize {
        self.turns.len()
    }

    /// num turns plus queued
    pub fn all_turns_count(&self) -> usize {
        self.turns_count() + self.queued_turns.len()
    }
}
