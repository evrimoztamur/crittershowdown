use std::collections::HashMap;

use nalgebra::{vector, Point2, Vector2};
use rapier2d::{dynamics::RigidBodyHandle};
use serde::{Deserialize, Serialize};

use crate::{Bug, BugData, Physics, Result, Team};

/// Game structure.
#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(skip)]
    physics: Physics,
    bugs: HashMap<RigidBodyHandle, BugData>,
}

impl Game {
    /// Initializes a new [`Game`].
    pub fn new() -> Game {
        let mut game = Game {
            physics: Physics::from_settings(),
            bugs: HashMap::new(),
        };

        for i in 0..16 {
            game.insert_bug(
                vector![0.0 + (i as f32).cos() * 2.0, 0.0 + (i as f32).sin() * 2.0],
                BugData::new(crate::BugSort::WaterBeetle, Team::Blue),
            );
        }

        game
    }

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
    pub fn intersecting_bug(&self, point: Point2<f32>) -> Option<Bug> {
        if let Some((collider_handle, _)) = self.physics.intersecting_collider(point) {
            if let Some(collider) = self.physics.collider_set.get(collider_handle) {
                if let Some(collider_parent_handle) = collider.parent() {
                    if let Some(rigid_body) =
                        self.physics.rigid_body_set.get(collider_parent_handle)
                    {
                        if let Some(data) = self.bugs.get(&collider_parent_handle) {
                            Some(Bug { rigid_body, data })
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
    pub fn iter_bugs(&self) -> impl Iterator<Item = Bug> {
        self.physics
            .rigid_body_set
            .iter()
            .filter_map(|(rigid_body_handle, rigid_body)| {
                self.bugs
                    .get(&rigid_body_handle)
                    .and_then(|data| Some(Bug { rigid_body, data }))
            })
    }

    /// Inserts a new [`Bug`].
    pub fn insert_bug(&mut self, translation: Vector2<f32>, bug_data: BugData) -> RigidBodyHandle {
        let rigid_body_handle = self.physics.insert_bug(translation);

        self.bugs.insert(rigid_body_handle, bug_data);

        rigid_body_handle
    }
}
