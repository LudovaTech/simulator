use eframe::egui;

const FIELD_LENGH: u32 = 100;
const FIELD_DEPTH: u32 = 200;

struct MyApp {
    robot1_pos: egui::Pos2,
    robot2_pos: egui::Pos2,
    scale: f32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            robot1_pos: egui::pos2(100.0, 100.0),
            robot2_pos: egui::pos2(300.0, 100.0),
            scale: 1.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Dessiner le terrain
            self.draw_field(ui);
            // Dessiner les robots
            self.draw_robot(ui, self.robot1_pos, egui::Color32::from_rgb(255, 0, 0));
            self.draw_robot(ui, self.robot2_pos, egui::Color32::from_rgb(0, 0, 255));
            // Ajouter des boutons
            ui.horizontal(|ui| {
                if ui.button("Move Robot 1 Right").clicked() {
                    self.robot1_pos.x += 10.0;
                }
                if ui.button("Move Robot 2 Left").clicked() {
                    self.robot2_pos.x -= 10.0;
                }
            });
            ui.add(egui::Slider::new(&mut self.scale, 0.0..=5.0).text("Scale"));
        });
    }
}

trait DrawFieldObjects {
    fn draw_field(&self, ui: &mut egui::Ui) -> ();
    fn draw_robot(&self, ui: &mut egui::Ui, pos: egui::Pos2, color: egui::Color32) -> ();
}

impl DrawFieldObjects for MyApp {
    fn draw_field(&self, ui: &mut egui::Ui) {
        let field_rect = egui::Rect::from_min_size(
            egui::pos2(50.0, 50.0) * self.scale,
            egui::vec2(FIELD_LENGH as f32, FIELD_DEPTH as f32) * self.scale,
        );
        ui.painter()
            .rect_filled(field_rect, 0.0, egui::Color32::from_rgb(0, 128, 0));
    }

    fn draw_robot(&self, ui: &mut egui::Ui, pos: egui::Pos2, color: egui::Color32) {
        let robot_rect = egui::Rect::from_min_size(pos * self.scale, egui::vec2(15.0, 15.0) * self.scale);
        ui.painter()
            .rect_filled(robot_rect, 0.0, color);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Robot Analyzer",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}
