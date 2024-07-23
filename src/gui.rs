use crate::objects::{CircularDraw, Robot};

use crate::infos;
use crate::vector2::vector2;

const BUTTON_PANEL_WIDTH: f32 = 150.0;

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
            robot_a1: Robot::new(vector2(50.0, 50.0), egui::Color32::from_rgb(255, 255, 0)),
            robot_a2: Robot::new(vector2(50.0, 50.0), egui::Color32::from_rgb(0, 255, 255)),
            robot_b1: Robot::new(vector2(50.0, 50.0), egui::Color32::from_rgb(255, 0, 255)),
            robot_b2: Robot::new(
                vector2(50.0, 50.0),
                egui::Color32::from_rgb(255, 255, 255),
            ),
            scale: 1.0,
        }
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ui.max_rect();
            ui.horizontal(|ui| {
                ui.set_height(window_size.height());
                //buttons panel
                ui.vertical(|ui| {
                    ui.set_width(BUTTON_PANEL_WIDTH);
                    if ui.button("Move Robot A1 Right").clicked() {
                        self.robot_a1.pos.x += 10.0;
                    }
                    if ui.button("Move Robot 2 Left").clicked() {
                        self.robot_b1.pos.x -= 10.0;
                    }
                });

                // Paint zone
                ui.vertical(|ui| {
                    let available_size = ui.available_size();
                    let painter_rect = egui::Rect::from_min_size(
                        ui.min_rect().min,
                        egui::vec2(available_size.x, available_size.y),
                    );

                    let painter = ui.painter_at(painter_rect);

                    self.scale = (painter_rect.width() / (infos::FIELD_DEPTH as f32))
                        .min(painter_rect.height() / (infos::FIELD_WIDTH as f32));

                    // montre la zone du painter
                    // painter.rect_filled(painter_rect, 0.0, egui::Color32::BLUE);

                    self.draw_field(&painter, painter_rect.min.to_vec2());

                    for obj in [
                        &self.robot_a1,
                        &self.robot_a2,
                        &self.robot_b1,
                        &self.robot_b2,
                    ]
                    .into_iter()
                    {
                        self.draw_circular_obj(&painter, obj, painter_rect.min.to_vec2())
                    }
                });
            });
        });
    }
}

impl SimulatorApp {
    fn draw_field(&self, painter: &egui::Painter, offset: egui::Vec2) {
        let stroke: egui::Stroke = egui::Stroke::new(2.0 * self.scale, egui::Color32::WHITE);
        painter.rect_filled(
            egui::Rect::from_min_size(
                (egui::pos2(0.0, 0.0) * self.scale) + offset,
                egui::vec2(infos::FIELD_DEPTH as f32, infos::FIELD_WIDTH as f32) * self.scale,
            ),
            0.0,
            egui::Color32::from_rgb(0, 128, 0),
        );
        painter.hline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.x
                ..=(((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale)
                    + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.y,
            stroke,
        );
        painter.hline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.x
                ..=(((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale)
                    + offset.x,
            (((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale) + offset.y,
            stroke,
        );
        painter.vline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.y
                ..=(((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale)
                    + offset.y,
            stroke,
        );
        painter.vline(
            (((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale) + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) + offset.y
                ..=(((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * self.scale)
                    + offset.y,
            stroke,
        );

        let radius: f32 = (infos::ENBUT_RADIUS as f32) * self.scale;

        // left enbut
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x + ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale),
                    offset.y
                        + ((((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) as f32) / 2.0) * self.scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH as f32, infos::ENBUT_WIDTH as f32) * self.scale,
            ),
            egui::Rounding {
                nw: 0.0,
                ne: radius,
                sw: 0.0,
                se: radius,
            },
            stroke,
        );

        // right enbut
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x
                        + (((infos::FIELD_DEPTH
                            - infos::ENBUT_DEPTH
                            - infos::SPACE_BEFORE_LINE_SIDE) as f32)
                            * self.scale),
                    offset.y
                        + ((((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) as f32) / 2.0) * self.scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH as f32, infos::ENBUT_WIDTH as f32) * self.scale,
            ),
            egui::Rounding {
                nw: radius,
                ne: 0.0,
                sw: radius,
                se: 0.0,
            },
            stroke,
        );

        let goal_size = egui::vec2(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * self.scale) - stroke.width/2.0,
            (infos::GOAL_WIDTH as f32) * self.scale,
        );

        // yellow goal
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x,
                    offset.y
                        + (((infos::FIELD_WIDTH - infos::GOAL_WIDTH) as f32) / 2.0) * self.scale,
                ),
                goal_size,
            ),
            0.0,
            egui::Color32::YELLOW,
        );

        // blue goal
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x + ((infos::FIELD_DEPTH as f32) * self.scale) - goal_size.x,
                    offset.y
                        + (((infos::FIELD_WIDTH - infos::GOAL_WIDTH) as f32) / 2.0) * self.scale,
                ),
                goal_size,
            ),
            0.0,
            egui::Color32::BLUE,
        );
    }

    fn draw_circular_obj(&self, painter: &egui::Painter, obj: &dyn CircularDraw, offset: egui::Vec2) {
        painter.circle_filled(
            (obj.position() * self.scale) + offset,
            obj.radius() * self.scale,
            obj.color(),
        );
    }
}

pub fn start() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Robot Analyzer",
        options,
        Box::new(|_cc| Box::<SimulatorApp>::default()),
    )
}
