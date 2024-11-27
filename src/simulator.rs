
use std::collections::HashMap;
use crate::robot::RobotHandler;
use crossbeam::channel::Receiver;
use rapier2d::prelude::*;

pub struct SimulatorApp {
    // World (rapier) :
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: nalgebra::Vector2<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub physics_hooks: (),
    pub event_handler: ChannelEventCollector,
    pub collision_recv: Receiver<CollisionEvent>,
    pub contact_force_recv: Receiver<ContactForceEvent>,
    // Simulator :
    pub robots: [RobotHandler; 4],
}

impl Default for SimulatorApp {
    fn default() -> SimulatorApp {
        let (collision_sender, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_sender, contact_force_recv) = crossbeam::channel::unbounded();
        SimulatorApp {
            // World (rapier) :
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: ChannelEventCollector::new(collision_sender, contact_force_sender),
            collision_recv,
            contact_force_recv,
            // Simulator :
            robots: [
                RobotHandler::new('A', 1),
                RobotHandler::new('A', 2),
                RobotHandler::new('B', 1),
                RobotHandler::new('B', 2),
            ],
        }
    }
}

impl SimulatorApp {
    pub fn update(&mut self) {
        // physic step
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
            &self.physics_hooks,
            &self.event_handler,
        );
        // 
    }
}
