use nalgebra::{point, vector, Point2, Vector2};
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase},
    pipeline::PhysicsPipeline,
    prelude::{ColliderHandle, PointProjection, QueryFilter, QueryPipeline},
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
    /// TODO docs
    pub rigid_body_set: RigidBodySet,
    /// TODO docs
    pub collider_set: ColliderSet,
    query_pipeline: QueryPipeline,
}

impl Physics {
    /// Inserts a new [`RigidBody`] for a [`Bug`].
    pub fn insert_bug(&mut self, translation: Vector2<f32>, index: usize) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic()
            .ccd_enabled(true)
            .translation(translation)
            .linear_damping(1.5)
            .user_data(index as u128)
            .build();

        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let ball_body_handle = self.rigid_body_set.insert(rigid_body);

        self.collider_set
            .insert_with_parent(collider, ball_body_handle, &mut self.rigid_body_set);

        ball_body_handle
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
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    /// Retrieves the closest [`ColliderHandle`] which intersects with a [`Point2`].
    pub fn intersecting_collider(
        &self,
        point: Point2<f32>,
    ) -> Option<(ColliderHandle, PointProjection)> {
        let solid = true;
        let filter = QueryFilter::default();

        self.query_pipeline
            .project_point(
                &self.rigid_body_set,
                &self.collider_set,
                &point,
                solid,
                filter,
            )
            .map_or(None, |(collider_handle, point_projection)| {
                if point_projection.is_inside {
                    let a = &self.collider_set[collider_handle];
                    Some((collider_handle, point_projection))
                } else {
                    None
                }
            })
    }
}

impl Clone for Physics {
    fn clone(&self) -> Self {
        Self {
            physics_pipeline: PhysicsPipeline::default(),
            gravity: self.gravity.clone(),
            integration_parameters: self.integration_parameters.clone(),
            island_manager: self.island_manager.clone(),
            broad_phase: self.broad_phase.clone(),
            narrow_phase: self.narrow_phase.clone(),
            impulse_joint_set: self.impulse_joint_set.clone(),
            multibody_joint_set: self.multibody_joint_set.clone(),
            ccd_solver: self.ccd_solver.clone(),
            rigid_body_set: self.rigid_body_set.clone(),
            collider_set: self.collider_set.clone(),
            query_pipeline: self.query_pipeline.clone(),
        }
    }
}

impl Default for Physics {
    fn default() -> Physics {
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
        let query_pipeline = QueryPipeline::new();

        let mut physics = Physics {
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
            query_pipeline,
        };

        let map_width = 18.0;
        let map_height = 12.0;

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(map_width / 2.0, 0.5)
            .translation(vector![0.0, -map_height / 2.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(map_width / 2.0, 0.5)
            .translation(vector![0.0, map_height / 2.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(0.5, map_height / 2.0)
            .translation(vector![map_width / 2.0, 0.0])
            .build();
        physics.collider_set.insert(collider);

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(0.5, map_height / 2.0)
            .translation(vector![-map_width / 2.0, 0.0])
            .build();
        physics.collider_set.insert(collider);

        physics
    }
}
