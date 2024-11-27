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
