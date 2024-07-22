use robot::Robot;

use crate::infos;

mod robot;

struct SimulatorApp {
    robot_a1: Robot,
    robot_a2: Robot,
    robot_b1: Robot,
    robot_b2: Robot,
    scale: f32,
}

impl Default for SimulatorApp {
    fn default() -> Self {
        Self {
            robot_a1: Robot::new(egui::pos2(50.0, 50.0), egui::Color32::from_rgb(255, 255, 0)),
            robot_a2: Robot::new(egui::pos2(50.0, 50.0), egui::Color32::from_rgb(0, 255, 255)),
            robot_b1: Robot::new(egui::pos2(50.0, 50.0), egui::Color32::from_rgb(255, 0, 255)),
            robot_b2: Robot::new(
                egui::pos2(50.0, 50.0),
                egui::Color32::from_rgb(255, 255, 255),
            ),
            scale: 1.0,
        }
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_field(ui);
            for robot in [
                &self.robot_a1,
                &self.robot_a2,
                &self.robot_b1,
                &self.robot_b2,
            ]
            .into_iter()
            {
                self.draw_robot(ui, robot)
            }

            // Ajouter des boutons
            ui.horizontal(|ui| {
                if ui.button("Move Robot A1 Right").clicked() {
                    self.robot_a1.pos.x += 10.0;
                }
                if ui.button("Move Robot 2 Left").clicked() {
                    self.robot_b1.pos.x -= 10.0;
                }
            });
            ui.add(egui::Slider::new(&mut self.scale, 0.0..=5.0).text("Scale"));
        });
    }
}

impl SimulatorApp {
    fn draw_field(&self, ui: &mut egui::Ui) {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0) * self.scale,
                egui::vec2(infos::FIELD_DEPTH as f32, infos::FIELD_WIDTH as f32) * self.scale,
            ),
            0.0,
            egui::Color32::from_rgb(0, 128, 0),
        );
        ui.painter().hline(
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale
                ..=((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale,
            egui::Stroke::new(2.0 * self.scale, egui::Color32::WHITE),
        );
        ui.painter().hline(
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale
                ..=((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            ((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            egui::Stroke::new(2.0 * self.scale, egui::Color32::WHITE),
        );
        ui.painter().vline(
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale,
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale
                ..=((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            egui::Stroke::new(2.0 * self.scale, egui::Color32::WHITE),
        );
        ui.painter().vline(
            ((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            (infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale
                ..=((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale,
            egui::Stroke::new(2.0 * self.scale, egui::Color32::WHITE),
        );
    }

    fn draw_robot(&self, ui: &mut egui::Ui, robot: &Robot) {
        ui.painter().circle_filled(
            robot.pos * self.scale,
            (infos::ROBOT_RADIUS as f32) * self.scale,
            robot.color,
        );
    }
}

fn egui_pos_to_our_pos(egui_pos: egui::Pos2) -> egui::Pos2 {
    (egui_pos
        - egui::pos2(
            (infos::FIELD_DEPTH as f32) / 2.0,
            (infos::FIELD_WIDTH as f32) / 2.0,
        ))
    .to_pos2()
}

fn our_pos_to_egui_pos(our_pos: egui::Pos2) -> egui::Pos2 {
    our_pos
        + egui::pos2(
            (infos::FIELD_DEPTH as f32) / 2.0,
            (infos::FIELD_WIDTH as f32) / 2.0,
        )
        .to_vec2()
}

pub fn start() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Robot Analyzer",
        options,
        Box::new(|_cc| Box::<SimulatorApp>::default()),
    )
}
