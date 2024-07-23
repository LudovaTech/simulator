use vector2d::Vector2D;

pub type Vector2 = Vector2D<f32>;

pub trait EguiConvertCompatibility {
    fn to_egui_vec2(&self) -> egui::Vec2;
    fn to_egui_pos2(&self) -> egui::Pos2;
}


impl EguiConvertCompatibility for Vector2 {
    fn to_egui_vec2(&self) -> egui::Vec2 {
        egui::Vec2 { x: self.x, y: self.y}
    }
    fn to_egui_pos2(&self) -> egui::Pos2 {
        egui::Pos2 { x: self.x, y: self.y}
    }
}

pub fn vector2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}