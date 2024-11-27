use crate::{infos, robot::{RobotBuilder, RobotHandler}};
use crossbeam::channel::Receiver;
use nalgebra::Vector2;
use rapier2d::prelude::*;
use std::collections::HashMap;

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
    pub robot_to_rigid_body_handle: HashMap<RobotHandler, RigidBodyHandle>,
    pub collider_to_robot_handle: HashMap<ColliderHandle, RobotHandler>,
}

impl Default for SimulatorApp {
    fn default() -> Self {
        SimulatorApp::new(
            [
                RobotBuilder{
                    team_name: 'A',
                    robot_number: 1,
                    initial_position: vector!(50.0, 50.0),
                    friction: infos::ROBOT_FRICTION,
                    mass: infos::ROBOT_MASS,
                    radius: infos::ROBOT_RADIUS,
                },
                RobotBuilder{
                    team_name: 'A',
                    robot_number: 2,
                    initial_position: vector!(50.0, 75.0),
                    friction: infos::ROBOT_FRICTION,
                    mass: infos::ROBOT_MASS,
                    radius: infos::ROBOT_RADIUS,
                },
                RobotBuilder{
                    team_name: 'B',
                    robot_number: 1,
                    initial_position: vector!(50.0, 100.0),
                    friction: infos::ROBOT_FRICTION,
                    mass: infos::ROBOT_MASS,
                    radius: infos::ROBOT_RADIUS,
                },
                RobotBuilder{
                    team_name: 'B',
                    robot_number: 2,
                    initial_position: vector!(50.0, 125.0),
                    friction: infos::ROBOT_FRICTION,
                    mass: infos::ROBOT_MASS,
                    radius: infos::ROBOT_RADIUS,
                },
            ]
        )
    }
}

impl SimulatorApp {
    pub fn new(robots_builders: [RobotBuilder; 4]) -> SimulatorApp {
        let robot_handlers: [RobotHandler; 4] = [
            robots_builders[0].to_robot_handle(),
            robots_builders[1].to_robot_handle(),
            robots_builders[2].to_robot_handle(),
            robots_builders[3].to_robot_handle(),
        ];
        let (collision_sender, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_sender, contact_force_recv) = crossbeam::channel::unbounded();
        let mut sym = SimulatorApp {
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
            robots: robot_handlers,
            robot_to_rigid_body_handle: HashMap::new(),
            collider_to_robot_handle: HashMap::new(),
        };

        // Construct HashMaps
        for robot_builder in robots_builders {
            let rigid_body_handle = sym.create_rigid_body(&robot_builder);
            sym.robot_to_rigid_body_handle.insert(robot_builder.to_robot_handle(), rigid_body_handle);
            let collider_handle = sym.create_collider(&robot_builder, rigid_body_handle);
            sym.collider_to_robot_handle.insert(collider_handle, robot_builder.to_robot_handle());
        }

        sym.build_field_colliders();
        sym
    }
}

impl SimulatorApp {
    fn create_rigid_body(&mut self, robot_builder: &RobotBuilder) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .translation(robot_builder.initial_position)
            .build();
        self.rigid_body_set.insert(body)
    }

    fn create_collider(&mut self, robot_builder: &RobotBuilder, rigid_body_handle: RigidBodyHandle) -> ColliderHandle {
        let collider = ColliderBuilder::ball(robot_builder.radius)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .mass(robot_builder.mass)
            .friction(robot_builder.friction)
            .build();
        self.collider_set.insert_with_parent(collider, rigid_body_handle, &mut self.rigid_body_set)
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

    pub fn position_of(&self, robot_handle: &RobotHandler) -> Vector2<f32>{
        self.rigid_body_set[self.robot_to_rigid_body_handle[robot_handle]].position().translation.vector
    }
}


impl SimulatorApp {// TODO refactor plus joliment
    fn build_field_colliders(&mut self) {
        let front = ColliderBuilder::cuboid(infos::FIELD_DEPTH, 1.0)
            .build();
        self.collider_set.insert(front);

        let bottom = ColliderBuilder::cuboid(infos::FIELD_DEPTH, 1.0)
            .translation(vector![0.0, infos::FIELD_WIDTH])
            .build();
        self.collider_set.insert(bottom);

        let left = ColliderBuilder::cuboid(1.0, infos::FIELD_WIDTH)
            .build();
        self.collider_set.insert(left);

        let right = ColliderBuilder::cuboid(1.0, infos::FIELD_WIDTH)
            .translation(vector![infos::FIELD_DEPTH, 0.0])
            .build();
        self.collider_set.insert(right);

        let goal_left_up = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![0.0, (infos::FIELD_WIDTH / 2.0) - (infos::GOAL_WIDTH /2.0)])
            .build();
        self.collider_set.insert(goal_left_up);

        let goal_left_down = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![0.0, (infos::FIELD_WIDTH / 2.0) + (infos::GOAL_WIDTH /2.0)])
            .build();
        self.collider_set.insert(goal_left_down);

        let goal_right_up = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![infos::FIELD_DEPTH, (infos::FIELD_WIDTH / 2.0) - (infos::GOAL_WIDTH /2.0)])
            .build();
        self.collider_set.insert(goal_right_up);

        let goal_right_down = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![infos::FIELD_DEPTH, (infos::FIELD_WIDTH / 2.0) + (infos::GOAL_WIDTH /2.0)])
            .build();
        self.collider_set.insert(goal_right_down);
    }
}