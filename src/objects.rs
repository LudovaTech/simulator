use crate::{
    infos,
    vector2::{vector2, EguiConvertCompatibility, Vector2},
};

pub trait Movable {
    fn velocity(&self) -> Vector2;
    fn apply_velocity(&mut self);
}

pub trait PointDraw {
    fn position(&self) -> egui::Pos2;
    fn color(&self) -> egui::Color32;
}

pub trait CircularDraw: PointDraw {
    fn radius(&self) -> f32;
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

impl PointDraw for Robot {
    fn position(&self) -> egui::Pos2 {
        self.pos.to_egui_pos2()
    }
    fn color(&self) -> egui::Color32 {
        self.color
    }
}

impl CircularDraw for Robot {
    fn radius(&self) -> f32 {
        infos::ROBOT_RADIUS as f32
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

impl PointDraw for Ball {
    fn position(&self) -> egui::Pos2 {
        self.pos.to_egui_pos2()
    }
    fn color(&self) -> egui::Color32 {
        self.color
    }
}

impl CircularDraw for Ball {
    fn radius(&self) -> f32 {
        infos::BALL_RADIUS as f32
    }
}