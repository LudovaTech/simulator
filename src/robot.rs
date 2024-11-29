use nalgebra::Vector2;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RobotHandler {
    team_name: char,
    robot_number: u8,
}

impl RobotHandler {
    pub fn new(team_name: char, robot_number: u8) -> RobotHandler {
        RobotHandler {
            team_name,
            robot_number,
        }
    }
}

pub struct RobotBuilder {
    pub team_name: char,
    pub robot_number: u8,
    pub initial_position: Vector2<f32>,
    pub friction: f32,
    pub linear_damping: f32,
    pub restitution: f32,
    pub mass: f32,
    pub radius: f32,
}

impl RobotBuilder {
    pub fn to_robot_handle(&self) -> RobotHandler {
        RobotHandler {
            team_name: self.team_name,
            robot_number: self.robot_number,
        }
    }
}