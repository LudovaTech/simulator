use nalgebra::{Complex, Point2, Vector2};
use rerun::external::egui;

pub trait EguiConvertCompatibility {
    fn to_egui_vec2(&self) -> egui::Vec2;
    fn to_egui_pos2(&self) -> egui::Pos2;
}

impl EguiConvertCompatibility for Vector2<f32> {
    fn to_egui_vec2(&self) -> egui::Vec2 {
        egui::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
    fn to_egui_pos2(&self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl EguiConvertCompatibility for Point2<f32> {
    fn to_egui_vec2(&self) -> egui::Vec2 {
        egui::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
    fn to_egui_pos2(&self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl EguiConvertCompatibility for Complex<f32> {
    fn to_egui_vec2(&self) -> egui::Vec2 {
        egui::Vec2 {
            x: self.re,
            y: self.im,
        }
    }
    fn to_egui_pos2(&self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.re,
            y: self.im,
        }
    }
}