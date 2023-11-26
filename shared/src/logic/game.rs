use std::{
    collections::{HashMap, VecDeque},
    f64::consts::{PI, TAU},
};

use nalgebra::{vector, Point2, Vector2};
use rapier2d::{
    dynamics::{RigidBody, RigidBodyHandle},
    geometry::{Collider, ColliderHandle, ContactData},
};

use crate::{BugData, BugSort, Message, Physics, Player, PropData, Result, Team, Turn};

/// Game structure.
#[derive(Clone)]
pub struct Game {
    physics: Physics,
    bugs: HashMap<usize, BugData>,
    bug_handles: HashMap<usize, RigidBodyHandle>,
    props: HashMap<usize, PropData>,
    ticks: u64,
    turns: Vec<Turn>,
    queued_turns: VecDeque<Turn>,
    capture_radius: f32,
    capture_progress: i32,
    bug_collisions: Vec<((u128, u128), Point2<f32>)>,
    bug_impacts: Vec<((u128, u128), Point2<f32>)>,
}

impl Default for Game {
    fn default() -> Self {
        let mut game = Game {
            physics: Physics::default(),
            bugs: HashMap::new(),
            bug_handles: HashMap::new(),
            props: HashMap::new(),
            turns: Vec::new(),
            queued_turns: VecDeque::new(),
            ticks: 0,
            capture_radius: 4.0,
            capture_progress: 0,
            bug_collisions: Vec::new(),
            bug_impacts: Vec::new(),
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
                    0.0 + (net_offset).cos() * 8.0,
                    0.0 + (net_offset).sin() * 8.0
                ],
                match i % 3 {
                    0 => BugData::new(BugSort::Beetle, team),
                    1 => BugData::new(BugSort::Ladybug, team),
                    _ => BugData::new(BugSort::Ant, team),
                },
            );
        }

        for i in 0..24 {
            let offset = i;
            let arc_size = TAU / 16 as f64;
            let arc: f32 = arc_size as f32 * offset as f32;

            game.insert_prop(vector![
                0.0 + (arc * 1.0).cos() * 10.0,
                0.0 + (arc * 6.0).sin() * 10.0
            ]);
        }

        for i in 0..6 {
            let offset = i;
            let arc_size = TAU / 6 as f64;
            let arc: f32 = arc_size as f32 * offset as f32 + 3.141592653589793 / 6.0;

            game.insert_prop(vector![
                0.0 + (arc * 1.0).cos() * 6.0,
                0.0 + (arc * 1.0).sin() * 6.0
            ]);
        }

        for i in 0..4 {
            let offset = i;
            let arc_size = TAU / 4.0;
            let arc: f32 = arc_size as f32 * offset as f32 + 3.141592653589793 / 8.0;

            game.insert_prop(vector![
                0.0 + (arc * 1.0).cos() * 3.0,
                0.0 + (arc * 1.0).sin() * 3.0
            ]);
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
        self.ticks += 1;

        let turn_ticks = self.turn_ticks();
        let turn_tick_count = self.turn_tick_count();
        let turn_tick_count_half = self.turn_tick_count_half();

        if turn_ticks == 0 {
            // At each N second interval, check for queued turns (which are sent from the server
            if let Some(queued_turn) = self.queued_turns.pop_front() {
                if self.execute_turn(&queued_turn) {
                    self.tick_physics();
                }
            } else {
                // Do not progress ticks
                self.ticks -= 1;
            }
            // Do not act until available
        } else if turn_ticks < turn_tick_count_half {
            self.tick_physics();
        }
        if turn_ticks == turn_tick_count_half {
            self.tick_turn();
        }

        // Tick until we reach the next target
        if !self.queued_turns.is_empty() {
            self.tick();
        }
    }

    /// num turn ticks
    pub fn turn_ticks(&self) -> u64 {
        self.ticks % self.turn_tick_count()
    }

    /// percentage of turn passed
    pub fn turn_percentage_time(&self) -> f64 {
        self.turn_ticks() as f64 / self.turn_tick_count() as f64
    }

    /// Duration of the turn in seconds
    pub fn turn_duration(&self) -> u64 {
        16
    }

    /// num turn turn_tick_count
    pub fn turn_tick_count(&self) -> u64 {
        self.turn_duration() * 60
    }

    /// num turn turn_tick_count
    pub fn turn_tick_count_half(&self) -> u64 {
        4 * 60
    }

    /// num turn turn_tick_count
    pub fn turn_percentage_time_half(&self) -> f64 {
        self.turn_tick_count_half() as f64 / self.turn_tick_count() as f64
    }

    // /// target tick
    // pub fn target_tick(&self) -> u64 {
    //     ((self.all_turns_count() as f64) * 7.0 * 60.0).max(0.0) as u64
    // }

    /// force a subtick
    ///
    pub fn tick_turn(&mut self) {
        let mut tip = 0;

        for (rigid_body, bug_data) in self.iter_bugs() {
            if rigid_body.translation().magnitude() < self.capture_radius && bug_data.health() > 1 {
                match bug_data.team() {
                    Team::Red => tip += 1,
                    Team::Blue => tip -= 1,
                }
            }
        }

        for (_, bug_data) in self.bugs.iter_mut() {
            bug_data.add_health(1);
        }

        self.capture_progress += tip;
    }

    /// force a subtick
    pub fn tick_physics(&mut self) {
        self.physics.tick();

        self.bug_collisions = self.physics.bug_collisions();

        self.bug_impacts = Vec::new();

        for ((a, b), position) in self.bug_collisions.clone() {
            let (rb_a, bug_a) = self.get_bug(a as usize).unwrap();
            let (rb_b, bug_b) = self.get_bug(b as usize).unwrap();

            let max_linvel = rb_a.linvel().magnitude().max(rb_b.linvel().magnitude());

            if max_linvel > 2.0 && bug_a.team() != bug_b.team() {
                if rb_a.linvel().magnitude() > rb_b.linvel().magnitude() {
                    self.bug_impacts.push(((a, b), position));
                } else {
                    self.bug_impacts.push(((a, b), position));
                }
            }
        }

        for ((a, b), position) in self.bug_impacts.clone() {
            let (rb_a, bug_a) = self.get_bug_mut(a as usize).unwrap();
            bug_a.add_health(-1);

            let attacker_sort = *bug_a.sort();

            let (rb_b, bug_b) = self.get_bug_mut(b as usize).unwrap();
            bug_b.add_health(-1);

            if attacker_sort == BugSort::Ant {
                bug_b.add_health(-1);
            }
        }
    }

    /// bug collisions
    fn bug_collisions(&self) -> Vec<((u128, u128), Point2<f32>)> {
        self.bug_collisions.clone()
    }

    /// bug impacts
    pub fn bug_impacts(&self) -> Vec<((u128, u128), Point2<f32>)> {
        self.bug_impacts.clone()
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
            .filter_map(|(_rigid_body_handle, rigid_body)| {
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
            .filter_map(|(_rigid_body_handle, rigid_body)| {
                self.bugs
                    .get(&(rigid_body.user_data as usize))
                    .and_then(|data| Some((rigid_body, data)))
            })
    }

    /// Returns an iterator over all active [`Bugs`].
    pub fn iter_props(&self) -> impl Iterator<Item = (&Collider, &PropData)> {
        self.physics
            .collider_set
            .iter()
            .filter_map(|(_collider_handle, collider)| {
                self.props
                    .get(&(collider.user_data as usize))
                    .and_then(|data| Some((collider, data)))
            })
    }

    /// Returns an iterator over all active [`Bugs`].
    pub fn iter_propmuts(&mut self) -> impl Iterator<Item = (&mut Collider, &PropData)> {
        self.physics
            .collider_set
            .iter_mut()
            .filter_map(|(_collider_handle, collider)| {
                self.props
                    .get(&(collider.user_data as usize))
                    .and_then(|data| Some((collider, data)))
            })
    }

    /// Inserts a new [`Bug`].
    pub fn insert_prop(&mut self, translation: Vector2<f32>) -> (usize, ColliderHandle) {
        let prop_index = self.props.len() + 0xff;
        let collider_handle = self.physics.insert_prop(translation, prop_index);

        self.props.insert(prop_index, PropData {});

        (prop_index, collider_handle)
    }

    /// Inserts a new [`Bug`].
    pub fn insert_bug(
        &mut self,
        translation: Vector2<f32>,
        bug_data: BugData,
    ) -> (usize, RigidBodyHandle) {
        let bug_index = self.bugs.len() + 0x01;
        let rigid_body_handle = self
            .physics
            .insert_bug(translation, bug_index, *bug_data.sort());

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
                        if bug_data.team() == &player.team && bug_data.health() > 1 {
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

    /// diameter of the capture zone
    pub fn capture_progress(&self) -> f32 {
        self.capture_progress as f32 / self.bugs.len() as f32
    }

    /// cap rad
    pub fn capture_radius(&self) -> f32 {
        self.capture_radius
    }
}
