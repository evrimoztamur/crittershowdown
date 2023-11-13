use nalgebra::{vector, Vector2};
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase},
    pipeline::PhysicsPipeline,
};

/// Wrapper for rapier2d.
pub struct Physics {
    physics_pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    ball_body_handle: Option<RigidBodyHandle>,
}

impl Default for Physics {
    fn default() -> Physics {
        /* Create all structures necessary for the simulation. */
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let gravity = vector![0.0, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let ball_body_handle = None;

        Physics {
            physics_pipeline,
            gravity,
            integration_parameters,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            rigid_body_set,
            collider_set,
            ball_body_handle,
        }
    }
}

impl Physics {
    /// TODO docs
    pub fn from_settings() -> Physics {
        let mut physics = Physics::default();

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(10.0, 0.1)
            .translation(vector![0.0, -10.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(10.0, 0.1)
            .translation(vector![0.0, 10.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(0.1, 10.0)
            .translation(vector![10.0, 0.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(0.1, 10.0)
            .translation(vector![-10.0, 0.0])
            .build();
        physics.collider_set.insert(collider);

        // /* Create the ground. */
        // let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
        // physics.collider_set.insert(collider);

        for i in 0..100 {
            /* Create the bouncing ball. */
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(vector![4.0 + (i as f32).cos(), 4.0 + (i as f32).sin()])
                .linear_damping(0.5)
                .build();
            let collider = ColliderBuilder::ball(0.5).restitution(1.0).build();
            let ball_body_handle = physics.rigid_body_set.insert(rigid_body);
            physics.collider_set.insert_with_parent(
                collider,
                ball_body_handle,
                &mut physics.rigid_body_set,
            );
        }

        // physics.ball_body_handle = Some(ball_body_handle);

        physics
    }

    /// TODO docs
    pub fn tick(&mut self) {
        /* Run the game loop, stepping the simulation once per frame. */
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }
    /// TODO docs
    pub fn ball_positions(&self) -> Vec<Vector2<f32>> {
        self.rigid_body_set
            .iter()
            .map(|(rbh, rb)| rb.translation().scale(16.0) + vector![128.0, 128.0])
            .collect()
    }
}
