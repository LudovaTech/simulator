use crate::objects::{Ball, Drawable, Robot};

use crate::infos;
use crate::world::World;
use nalgebra::vector;

const BUTTON_PANEL_WIDTH: f32 = 150.0;

struct SimulatorApp {
    world: &'static mut World,
    robot_a1: Robot,
    robot_a2: Robot,
    robot_b1: Robot,
    robot_b2: Robot,
    ball: Ball,
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.world.step();
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ui.max_rect();
            ui.horizontal(|ui| {
                ui.set_height(window_size.height());
                //buttons panel
                ui.vertical(|ui| {
                    ui.set_width(BUTTON_PANEL_WIDTH);
                    if ui.button("Move Robot A1 Right").clicked() {
                        //self.robot_a1.move_base.position.x += 10.0;
                    }
                    if ui.button("Move Robot 2 Left").clicked() {
                        //self.robot_b1.move_base.position.x -= 10.0;
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

                    let scale: f32 = (painter_rect.width() / (infos::FIELD_DEPTH as f32))
                        .min(painter_rect.height() / (infos::FIELD_WIDTH as f32));

                    // montre la zone du painter
                    // painter.rect_filled(painter_rect, 0.0, egui::Color32::BLUE);

                    self.draw_field(&painter, painter_rect.min.to_vec2(), scale);

                    let circular_obj: [&Robot; 4] = [
                        &self.robot_a1,
                        &self.robot_a2,
                        &self.robot_b1,
                        &self.robot_b2,
                    ];

                    for obj in circular_obj.into_iter() {
                        obj.draw(self.world, &painter, painter_rect.min.to_vec2(), scale)
                    }
                    self.ball.draw(self.world, &painter, painter_rect.min.to_vec2(), scale)
                });
            });
        });
    }
}

impl SimulatorApp {
    fn new(world: &'static mut World) -> Self {
        let robot_a1 = Robot::new(world, vector!(50.0, 50.0), egui::Color32::from_rgb(255, 255, 0), 1500.0);
        let robot_a2 = Robot::new(world, vector!(50.0, 50.0), egui::Color32::from_rgb(0, 255, 255), 1500.0);
        let robot_b1 = Robot::new(world, vector!(50.0, 50.0), egui::Color32::from_rgb(255, 0, 255), 1500.0);
        let robot_b2 = Robot::new(world, vector!(50.0, 50.0), egui::Color32::from_rgb(255, 255, 255), 1500.0);
        let ball = Ball::new(world, vector!(100.0, 100.0), egui::Color32::from_rgb(255, 165, 0), 100.0);

        // Maintenant, créez et retournez l'application
        Self {
            world,
            robot_a1,
            robot_a2,
            robot_b1,
            robot_b2,
            ball,
        }
    }

    fn draw_field(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        let stroke: egui::Stroke = egui::Stroke::new(2.0 * scale, egui::Color32::WHITE);
        painter.rect_filled(
            egui::Rect::from_min_size(
                (egui::pos2(0.0, 0.0) * scale) + offset,
                egui::vec2(infos::FIELD_DEPTH as f32, infos::FIELD_WIDTH as f32) * scale,
            ),
            0.0,
            egui::Color32::from_rgb(0, 128, 0),
        );
        painter.hline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.x
                ..=(((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale)
                    + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.y,
            stroke,
        );
        painter.hline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.x
                ..=(((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale)
                    + offset.x,
            (((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale) + offset.y,
            stroke,
        );
        painter.vline(
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.y
                ..=(((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale)
                    + offset.y,
            stroke,
        );
        painter.vline(
            (((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale) + offset.x,
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) + offset.y
                ..=(((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) as f32) * scale)
                    + offset.y,
            stroke,
        );

        let radius: f32 = (infos::ENBUT_RADIUS as f32) * scale;

        // left enbut
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x + ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale),
                    offset.y + ((((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) as f32) / 2.0) * scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH as f32, infos::ENBUT_WIDTH as f32) * scale,
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
                            * scale),
                    offset.y + ((((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) as f32) / 2.0) * scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH as f32, infos::ENBUT_WIDTH as f32) * scale,
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
            ((infos::SPACE_BEFORE_LINE_SIDE as f32) * scale) - stroke.width / 2.0,
            (infos::GOAL_WIDTH as f32) * scale,
        );

        // yellow goal
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x,
                    offset.y + (((infos::FIELD_WIDTH - infos::GOAL_WIDTH) as f32) / 2.0) * scale,
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
                    offset.x + ((infos::FIELD_DEPTH as f32) * scale) - goal_size.x,
                    offset.y + (((infos::FIELD_WIDTH - infos::GOAL_WIDTH) as f32) / 2.0) * scale,
                ),
                goal_size,
            ),
            0.0,
            egui::Color32::BLUE,
        );
    }
}

pub fn start(world: &'static mut World) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Robot Analyzer",
        options,
        Box::new(|_cc| Box::new(SimulatorApp::new(world))),
    )
}
