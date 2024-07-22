

pub struct Robot {
    pub pos: egui::Pos2,
    pub color: egui::Color32,
}

impl Robot {
    pub fn new(pos: egui::Pos2, color: egui::Color32) -> Self {
        Self {
            pos,
            color
        }
    }
}