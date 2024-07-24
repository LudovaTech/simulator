use crate::{
    infos,
    vector2::{vector2, EguiConvertCompatibility, Vector2},
};

pub trait Movable {
    fn velocity(&self) -> Vector2;
    fn apply_velocity(&mut self);
}

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

////////////  ROBOT

pub struct Robot {
    pub pos: Vector2,
    pub color: egui::Color32,
}

impl Robot {
    pub fn new(pos: Vector2, color: egui::Color32) -> Self {
        Self { pos, color }
    }
}

impl Movable for Robot {
    fn velocity(&self) -> Vector2 {
        vector2(0.0, 0.0)
    }

    fn apply_velocity(&mut self) {
        self.pos += self.velocity();
    }
}

impl Drawable for Robot {
    fn draw(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            self.pos.to_egui_pos2(),
            infos::ROBOT_RADIUS as f32,
            self.color,
            offset,
            scale,
        );
    }
}

////////////  BALL

pub struct Ball {
    pub pos: Vector2,
    pub color: egui::Color32,
}

impl Ball {
    pub fn new(pos: Vector2, color: egui::Color32) -> Self {
        Self { pos, color }
    }
}

impl Movable for Ball {
    fn velocity(&self) -> Vector2 {
        vector2(0.0, 0.0)
    }

    fn apply_velocity(&mut self) {
        self.pos += self.velocity();
    }
}

impl Drawable for Ball {
    fn draw(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        draw_circular_generic(
            painter,
            self.pos.to_egui_pos2(),
            infos::BALL_RADIUS as f32,
            self.color,
            offset,
            scale,
        )
    }
}
