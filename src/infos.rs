/// Constantes définissant le terrain
pub const FIELD_WIDTH: f32 = 182.0;
pub const FIELD_DEPTH: f32 = 243.0;
pub const SPACE_BEFORE_LINE_SIDE: f32 = 12.0;
pub const GOAL_WIDTH: f32 = 60.0;
pub const ENBUT_DEPTH: f32 = 30.0;
pub const ENBUT_WIDTH: f32 = 75.0;
pub const ENBUT_RADIUS: f32 = 10.0;
pub const BORDER_RESTITUTION: f32 = 0.1;

// Positions pour le départ
pub const START_POS_ALIGNED_X: f32 = 25.0;
pub const START_POS_ALIGNED_Y: f32 = 25.0;

/// Constantes pour le robot
pub const ROBOT_RADIUS: f32 = 9.0;
pub const ROBOT_FRICTION: f32 = 0.0; // Friction lors d'un contact
pub const ROBOT_LINEAR_DAMPING: f32 = 0.5; // Friction qui s'applique tout le temps comme la résistance de l'air
pub const ROBOT_ANGULAR_DAMPING: f32 = 0.5; // De même mais pour la rotation
pub const ROBOT_RESTITUTION: f32 = 0.1; // Elasticité, restitution de la force de collision
pub const ROBOT_MASS: f32 = 10.0;

/// Constantes pour la balle
pub const BALL_RADIUS: f32 = 2.0;
pub const BALL_FRICTION: f32 = 0.0; // Friction lors d'un contact
pub const BALL_LINEAR_DAMPING: f32 = 0.5; // Friction qui s'applique tout le temps comme la résistance de l'air
pub const BALL_ANGULAR_DAMPING: f32 = 0.5; // De même mais pour la rotation
pub const BALL_RESTITUTION: f32 = 0.1; // Elasticité, restitution de la force de collision
pub const BALL_MASS: f32 = 10.0;

/// Constantes de jeu
pub const NB_MIN_TICK_BETWEEN_KICKS: u64 = 500;
pub const POWER_SPEED: f32 = 20.0;
pub const DISTANCE_MIN_KICKER_BALL: f32 = 5.0; // depuis le bord du robot
pub const KICK_POWER: f32 = 75.0;
pub const ROTATION_SPEED: f32 = 5.0_f32.to_radians();
pub const ROTATION_MAX_SPEED: f32 = 70.0_f32.to_radians();
pub const ROTATION_DELTA: f32 = 5.0_f32.to_radians();
pub const ROTATION_AUTO_DECREASE_RATIO: f32 = 0.9;