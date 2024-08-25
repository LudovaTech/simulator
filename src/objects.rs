//! Use Composition over inheritance in Rust !

use crate::{
    infos,
    vector2::{vector2, EguiConvertCompatibility, Vector2},
};

pub trait Drawable {
    fn draw(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32);
}

//////////// FUNCTIONS

fn draw_circular_generic(
    painter: &egui::Painter,
    pos: egui::Pos2,
    radius: f32,
    color: egui::Color32,
    offset: egui::Vec2,
    scale: f32,
) {
    painter.circle_filled((pos * scale) + offset, radius * scale, color);
}

//////////// CIRCULARMOVEBASE

///TODO, should be private
pub struct CircularMoveBase {
    pub position: Vector2,
    pub velocity: Vector2,
    pub mass: f32,
    pub radius: f32,
    pub restitution: f32,
}

impl CircularMoveBase {
    fn update_position(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }

    fn collides(&self, other: &CircularMoveBase) -> bool {
        let distance = (self.position - other.position).length();
        distance <= self.radius + other.radius
    }

    fn handle_collision(&mut self, other: &mut CircularMoveBase) {
        let normal = (other.position - self.position).normalise();

        let relative_velocity = other.velocity - self.velocity;
        let vel_along_normal = Vector2::dot(relative_velocity, normal);

        if vel_along_normal > 0.0 {
            return;
        }

        let j = -(1.0 + (self.restitution + other.restitution) / 2.0) * vel_along_normal;
        let j = j / (1.0 / self.mass + 1.0 / other.mass);

        let impulse = normal * j;

        self.velocity.x -= impulse.x / self.mass;
        self.velocity.y -= impulse.y / self.mass;

        other.velocity.x += impulse.x / other.mass;
        other.velocity.y += impulse.y / other.mass;
    }
}

////////////  ROBOT

pub struct Robot {
    pub move_base: CircularMoveBase,
    pub color: egui::Color32,
}

impl Robot {
    pub fn new(pos: Vector2, color: egui::Color32, mass: f32) -> Self {
        let move_base = CircularMoveBase {
            position: pos,
            velocity: Vector2 { x: 0.0, y: 0.0 },
            mass,
            radius: infos::ROBOT_RADIUS,
            restitution: 1.0,
        };
        Self {
            move_base,
            color,
        }
    }
}

impl Drawable for Robot {
    fn draw(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            self.move_base.position.to_egui_pos2(),
            infos::ROBOT_RADIUS,
            self.color,
            offset,
            scale,
        );
    }
}

////////////  BALL

pub struct Ball {
    pub move_base: CircularMoveBase,
    pub color: egui::Color32,
}

impl Ball {
    pub fn new(pos: Vector2, color: egui::Color32, mass: f32) -> Self {
        let move_base = CircularMoveBase {
            position: pos,
            velocity: Vector2 { x: 0.0, y: 0.0 },
            mass,
            radius: infos::ROBOT_RADIUS,
            restitution: 1.0,
        };
        Self { move_base, color }
    }
}

impl Drawable for Ball {
    fn draw(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            self.move_base.position.to_egui_pos2(),
            infos::BALL_RADIUS,
            self.color,
            offset,
            scale,
        )
    }
}
