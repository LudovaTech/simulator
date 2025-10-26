use nalgebra::Vector2;

use crate::infos;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RobotHandler {
    team_name: String,
    robot_number: u8,
}

impl RobotHandler {
    pub fn new(team_name: &str, robot_number: u8) -> RobotHandler {
        RobotHandler {
            team_name: team_name.to_owned(),
            robot_number,
        }
    }
}

impl std::fmt::Display for RobotHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_robot{}", self.team_name, self.robot_number)
    }
}

pub struct RobotBuilder {
    pub team_name: String,
    pub robot_number: u8,
    pub initial_position: Vector2<f32>,
    pub friction: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub restitution: f32,
    pub mass: f32,
    pub radius: f32,
}

impl RobotBuilder {
    pub fn to_robot_handle(&self) -> RobotHandler {
        RobotHandler {
            team_name: self.team_name.clone(),
            robot_number: self.robot_number,
        }
    }

    pub fn from_basic_robot(
        team_name: &str,
        robot_number: u8,
        initial_position: Vector2<f32>,
    ) -> Self {
        RobotBuilder {
            team_name: team_name.to_owned(),
            robot_number,
            initial_position,
            friction: infos::ROBOT_FRICTION,
            linear_damping: infos::ROBOT_LINEAR_DAMPING,
            angular_damping: infos::ROBOT_ANGULAR_DAMPING,
            restitution: infos::ROBOT_RESTITUTION,
            mass: infos::ROBOT_MASS,
            radius: infos::ROBOT_RADIUS,
        }
    }
}
