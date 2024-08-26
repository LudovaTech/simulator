//! Use Composition over inheritance in Rust !

use crate::{infos, vector_improver::EguiConvertCompatibility};
use rapier2d::prelude::*;
use nalgebra::Vector2;

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
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
    pub radius: f32,
    pub restitution: f32,
}

impl CircularMoveBase {
    fn update_position(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }

    fn collides(&self, other: &CircularMoveBase) -> bool {
        todo!()
    }

    fn handle_collision(&mut self, other: &mut CircularMoveBase) {
        todo!()
    }
}

////////////  ROBOT

pub struct Robot {
    pub move_base: CircularMoveBase,
    pub color: egui::Color32,
}

impl Robot {
    pub fn new(pos: Vector2<f32>, color: egui::Color32, mass: f32) -> Self {
        let move_base = CircularMoveBase {
            position: pos,
            velocity: Vector2::zeros(),
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
    pub fn new(pos: Vector2<f32>, color: egui::Color32, mass: f32) -> Self {
        let move_base = CircularMoveBase {
            position: pos,
            velocity: Vector2::zeros(),
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
