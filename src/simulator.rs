use crate::{
    infos,
    robot::{RobotBuilder, RobotHandler},
    game_referee::GameReferee,
};
use crossbeam::channel::Receiver;
use nalgebra::Vector2;
use rapier2d::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum FieldWallKind {
    Top,
    Bottom,
    Left,
    Right,
    GoalLeftUp,
    GoalLeftDown,
    GoalRightUp,
    GoalRightDown,
}

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
    pub game_referee: GameReferee,
    pub ball_rigid_body_handle: RigidBodyHandle,
    pub ball_collider_handle: ColliderHandle,
    pub robots: [RobotHandler; 4],
    pub robot_to_rigid_body_handle: HashMap<RobotHandler, RigidBodyHandle>,
    pub collider_to_robot_handle: HashMap<ColliderHandle, RobotHandler>,
    pub collider_to_field_wall: HashMap<ColliderHandle, FieldWallKind>,
}

impl Default for SimulatorApp {
    fn default() -> Self {
        SimulatorApp::new([
            RobotBuilder {
                team_name: 'A',
                robot_number: 1,
                initial_position: vector!(50.0, 50.0),
                friction: infos::ROBOT_FRICTION,
                linear_damping: infos::ROBOT_LINEAR_DAMPING,
                angular_damping: infos::ROBOT_ANGULAR_DAMPING,
                restitution: infos::ROBOT_RESTITUTION,
                mass: infos::ROBOT_MASS,
                radius: infos::ROBOT_RADIUS,
            },
            RobotBuilder {
                team_name: 'A',
                robot_number: 2,
                initial_position: vector!(50.0, 75.0),
                friction: infos::ROBOT_FRICTION,
                linear_damping: infos::ROBOT_LINEAR_DAMPING,
                angular_damping: infos::ROBOT_ANGULAR_DAMPING,
                restitution: infos::ROBOT_RESTITUTION,
                mass: infos::ROBOT_MASS,
                radius: infos::ROBOT_RADIUS,
            },
            RobotBuilder {
                team_name: 'B',
                robot_number: 1,
                initial_position: vector!(50.0, 100.0),
                friction: infos::ROBOT_FRICTION,
                linear_damping: infos::ROBOT_LINEAR_DAMPING,
                angular_damping: infos::ROBOT_ANGULAR_DAMPING,
                restitution: infos::ROBOT_RESTITUTION,
                mass: infos::ROBOT_MASS,
                radius: infos::ROBOT_RADIUS,
            },
            RobotBuilder {
                team_name: 'B',
                robot_number: 2,
                initial_position: vector!(50.0, 125.0),
                friction: infos::ROBOT_FRICTION,
                linear_damping: infos::ROBOT_LINEAR_DAMPING,
                angular_damping: infos::ROBOT_ANGULAR_DAMPING,
                restitution: infos::ROBOT_RESTITUTION,
                mass: infos::ROBOT_MASS,
                radius: infos::ROBOT_RADIUS,
            },
        ])
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
        let mut sim = SimulatorApp {
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
            game_referee: GameReferee::default(),
            ball_rigid_body_handle: RigidBodyHandle::invalid(),
            ball_collider_handle: ColliderHandle::invalid(),
            robots: robot_handlers,
            robot_to_rigid_body_handle: HashMap::new(),
            collider_to_robot_handle: HashMap::new(),
            collider_to_field_wall: HashMap::new(),
        };

        // Replace the invalid handles of the ball with the true values
        sim.ball_rigid_body_handle = sim.create_ball_rigid_body();
        sim.ball_collider_handle = sim.create_ball_collider(sim.ball_rigid_body_handle);

        // Construct HashMaps
        for robot_builder in robots_builders {
            let rigid_body_handle = sim.create_rigid_body(&robot_builder);
            sim.robot_to_rigid_body_handle
                .insert(robot_builder.to_robot_handle(), rigid_body_handle);
            let collider_handle = sim.create_collider(&robot_builder, rigid_body_handle);
            sim.collider_to_robot_handle
                .insert(collider_handle, robot_builder.to_robot_handle());
        }

        sim.build_field_colliders();
        sim
    }
}

impl SimulatorApp {
    fn create_rigid_body(&mut self, robot_builder: &RobotBuilder) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .linear_damping(robot_builder.linear_damping)
            .angular_damping(robot_builder.angular_damping)
            .translation(robot_builder.initial_position)
            .build();
        self.rigid_body_set.insert(body)
    }

    fn create_collider(
        &mut self,
        robot_builder: &RobotBuilder,
        rigid_body_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::ball(robot_builder.radius)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .mass(robot_builder.mass)
            .friction(robot_builder.friction)
            .restitution(robot_builder.restitution)
            .build();
        self.collider_set
            .insert_with_parent(collider, rigid_body_handle, &mut self.rigid_body_set)
    }

    fn create_ball_rigid_body(&mut self) -> RigidBodyHandle {
        let body = RigidBodyBuilder::dynamic()
            .linear_damping(infos::BALL_LINEAR_DAMPING)
            .angular_damping(infos::BALL_ANGULAR_DAMPING)
            .translation(Vector2::new(infos::FIELD_DEPTH/2.0, infos::FIELD_WIDTH/2.0))
            .build();
        self.rigid_body_set.insert(body)
    }

    fn create_ball_collider(&mut self, rigid_body_handle: RigidBodyHandle) -> ColliderHandle {
        let collider = ColliderBuilder::ball(infos::BALL_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .mass(infos::BALL_MASS)
            .friction(infos::BALL_FRICTION)
            .restitution(infos::BALL_RESTITUTION)
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
        self.process_collisions();
    }

    #[inline]
    pub fn position_of_ball(&self) -> Vector2<f32> {
        self.rigid_body_set[self.ball_rigid_body_handle]
            .position()
            .translation
            .vector
    }

    #[inline]
    pub fn position_of(&self, robot_handle: &RobotHandler) -> Vector2<f32> {
        self.rigid_body_set[self.robot_to_rigid_body_handle[robot_handle]]
            .position()
            .translation
            .vector
    }

    #[inline]
    pub fn rotation_of(&self, robot_handle: &RobotHandler) -> nalgebra::Unit<nalgebra::Complex<f32>> {
        self.rigid_body_set[self.robot_to_rigid_body_handle[robot_handle]]
            .rotation()
            .clone()
    }

    pub fn process_collisions(&self) {
        while let Ok(collision_event) = self.collision_recv.try_recv() {
            let try_robot_for_1 = self.collider_to_robot_handle.get(&collision_event.collider1());
            let try_robot_for_2 = self.collider_to_robot_handle.get(&collision_event.collider2());

            // Cas entre deux robots
            if let Some(robot1) = try_robot_for_1 {
                if let Some(robot2) = try_robot_for_2 {
                    println!("{} touched {}", robot1, robot2);
                    return;
                }
            }

            let try_wall_for_1 = self.collider_to_field_wall.get(&collision_event.collider1());
            let try_wall_for_2 = self.collider_to_field_wall.get(&collision_event.collider2());

            // Cas entre un robot et un mur
            if let Some(robot1) = try_robot_for_1 {
                if let Some(wall2) = try_wall_for_2 {
                    println!("{} touched {:?}", robot1, wall2);
                    return;
                }
            }
            if let Some(robot2) = try_robot_for_2 {
                if let Some(wall1) = try_wall_for_1 {
                    println!("{} touched {:?}", robot2, wall1);
                    return;
                }
            }

            let try_ball_for_1 = collision_event.collider1() == self.ball_collider_handle;
            let try_ball_for_2 = collision_event.collider2() == self.ball_collider_handle;

            // Cas ball/robot et ball/mur
            if try_ball_for_1 {
                if let Some(robot2) = try_robot_for_2 {
                    println!("ball touched {}", robot2);
                    return;
                }
                if let Some(wall2) = try_wall_for_2 {
                    println!("ball touched {:?}", wall2);
                    return;
                }
            }
            if try_ball_for_2 {
                if let Some(robot1) = try_robot_for_1 {
                    println!("ball touched {}", robot1);
                    return;
                }
                if let Some(wall1) = try_wall_for_1 {
                    println!("ball touched {:?}", wall1);
                    return;
                }
            }

            println!("Unknown collision : {:?} with {:?}", collision_event.collider1(), collision_event.collider2());
            dbg!(try_robot_for_1);
            dbg!(try_robot_for_2);
            dbg!(try_wall_for_1);
            dbg!(try_wall_for_2);
        }
    }

}

impl SimulatorApp {
    // TODO refactor plus joliment
    fn build_field_colliders(&mut self) {
        let up = ColliderBuilder::cuboid(infos::FIELD_DEPTH, 1.0)
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(up);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::Top);

        let bottom = ColliderBuilder::cuboid(infos::FIELD_DEPTH, 1.0)
            .translation(vector![0.0, infos::FIELD_WIDTH])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(bottom);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::Bottom);

        let left = ColliderBuilder::cuboid(1.0, infos::FIELD_WIDTH)
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(left);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::Left);

        let right = ColliderBuilder::cuboid(1.0, infos::FIELD_WIDTH)
            .translation(vector![infos::FIELD_DEPTH, 0.0])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(right);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::Right);

        let goal_left_up = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![
                0.0,
                (infos::FIELD_WIDTH / 2.0) - (infos::GOAL_WIDTH / 2.0)
            ])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(goal_left_up);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::GoalLeftUp);

        let goal_left_down = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![
                0.0,
                (infos::FIELD_WIDTH / 2.0) + (infos::GOAL_WIDTH / 2.0)
            ])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(goal_left_down);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::GoalLeftDown);

        let goal_right_up = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![
                infos::FIELD_DEPTH,
                (infos::FIELD_WIDTH / 2.0) - (infos::GOAL_WIDTH / 2.0)
            ])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(goal_right_up);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::GoalRightUp);

        let goal_right_down = ColliderBuilder::cuboid(infos::SPACE_BEFORE_LINE_SIDE, 1.0)
            .translation(vector![
                infos::FIELD_DEPTH,
                (infos::FIELD_WIDTH / 2.0) + (infos::GOAL_WIDTH / 2.0)
            ])
            .restitution(infos::BORDER_RESTITUTION)
            .build();
        let collider_handle = self.collider_set.insert(goal_right_down);
        self.collider_to_field_wall.insert(collider_handle, FieldWallKind::GoalRightDown);
    }
}
