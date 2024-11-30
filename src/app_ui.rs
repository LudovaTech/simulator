use std::collections::HashMap;

use nalgebra::vector;

use crate::{
    infos, robot::RobotHandler, simulator::SimulatorApp, vector_converter::EguiConvertCompatibility,
};

const BUTTON_PANEL_WIDTH: f32 = 150.0;

pub trait AppContainer: Default {
    fn start(self);
}

pub struct NoUIContainer {
    pub simulation: SimulatorApp,
}

impl AppContainer for NoUIContainer {
    fn start(mut self) {
        loop {
            self.simulation.update();
        }
    }
}

impl Default for NoUIContainer {
    fn default() -> Self {
        NoUIContainer {
            simulation: SimulatorApp::default(),
        }
    }
}

pub struct AppUIContainer {
    pub simulation: SimulatorApp,
    pub robot_handle_to_color: HashMap<RobotHandler, egui::Color32>,
}

impl AppContainer for AppUIContainer {
    fn start(self) {
        let options = eframe::NativeOptions::default();
        let _ = eframe::run_native("Simulator", options, Box::new(|_cc| Box::new(self)));
    }
}

impl Default for AppUIContainer {
    fn default() -> Self {
        let simulation = SimulatorApp::default();
        let mut robot_handle_to_color = HashMap::new();
        robot_handle_to_color.insert(simulation.robots[0], egui::Color32::YELLOW);
        robot_handle_to_color.insert(simulation.robots[1], egui::Color32::BLACK);
        robot_handle_to_color.insert(simulation.robots[2], egui::Color32::RED);
        robot_handle_to_color.insert(simulation.robots[3], egui::Color32::GREEN);
        AppUIContainer {
            simulation,
            robot_handle_to_color,
        }
    }
}

impl eframe::App for AppUIContainer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh simulation
        self.simulation.update();

        // Building UI :
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ui.max_rect();
            ui.horizontal(|ui| {
                ui.set_height(window_size.height());
                //buttons panel
                ui.vertical(|ui| {
                    ui.set_width(BUTTON_PANEL_WIDTH);

                    if ui.button("Move Robot A1 Right").clicked() {
                        self.simulation.rigid_body_set[self.simulation.robot_to_rigid_body_handle
                            [&RobotHandler::new('A', 1)]]
                            .apply_impulse(vector![100.0, 0.0], true);
                    }
                    if ui.button("Move Robot A1 Left").clicked() {
                        self.simulation.rigid_body_set[self.simulation.robot_to_rigid_body_handle
                            [&RobotHandler::new('A', 1)]]
                            .apply_impulse(vector![-100.0, 0.0], true);
                    }
                    if ui.button("Move Robot A1 Up").clicked() {
                        self.simulation.rigid_body_set[self.simulation.robot_to_rigid_body_handle
                            [&RobotHandler::new('A', 1)]]
                            .apply_impulse(vector![0.0, -100.0], true);
                    }
                    if ui.button("Move Robot A1 Down").clicked() {
                        self.simulation.rigid_body_set[self.simulation.robot_to_rigid_body_handle
                            [&RobotHandler::new('A', 1)]]
                            .apply_impulse(vector![0.0, 100.0], true);
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

                    let scale: f32 = (painter_rect.width() / infos::FIELD_DEPTH)
                        .min(painter_rect.height() / infos::FIELD_WIDTH);

                    let offset = painter_rect.min.to_vec2();

                    // montre la zone du painter
                    // painter.rect_filled(painter_rect, 0.0, egui::Color32::BLUE);

                    self.draw_field(&painter, offset, scale);

                    self.draw_ball(&painter, offset, scale);
                    for robot_handle in self.simulation.robots {
                        self.draw_robot(robot_handle, &painter, offset, scale);
                    }
                });
            });
        });
        // trigger keyboard keys
        ctx.input(|i| {
            if i.key_pressed(egui::Key::ArrowUp) {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new('A', 1)]]
                    .apply_impulse(vector![0.0, -100.0], true);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new('A', 1)]]
                    .apply_impulse(vector![0.0, 100.0], true);
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new('A', 1)]]
                    .apply_impulse(vector![100.0, 0.0], true);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new('A', 1)]]
                    .apply_impulse(vector![-100.0, 0.0], true);
            }
        });
        ctx.request_repaint();
    }
}

impl AppUIContainer {
    fn draw_ball(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        let pos_corrected = (self.simulation.position_of_ball().to_egui_pos2() * scale) + offset;
        let radius_corrected = infos::BALL_RADIUS * scale;
        painter.circle_filled(
            pos_corrected,
            radius_corrected, //TODO: doit être hérité et pas être mis en constante
            egui::Color32::LIGHT_RED,
        );
    }

    fn draw_robot(
        &self,
        robot_handle: RobotHandler,
        painter: &egui::Painter,
        offset: egui::Vec2,
        scale: f32,
    ) {
        let pos_corrected =
            (self.simulation.position_of(&robot_handle).to_egui_pos2() * scale) + offset;
        let radius_corrected = infos::ROBOT_RADIUS * scale;
        painter.circle_filled(
            pos_corrected,
            radius_corrected, //TODO: doit être hérité et pas être mis en constante
            self.robot_handle_to_color[&robot_handle],
        );

        // dribbler
        let corrected_angle = *self.simulation.rotation_of(&robot_handle);
        let dribbler_length = radius_corrected * 60.0 / 100.0;
        let dribbler_width = radius_corrected * 20.0 / 100.0;

        painter.line_segment(
            [
                (nalgebra::Complex::new(
                    -dribbler_length,
                    -radius_corrected + dribbler_width / 2.0,
                ) * corrected_angle)
                    .to_egui_pos2()
                    + pos_corrected.to_vec2(),
                (nalgebra::Complex::new(dribbler_length, -radius_corrected + dribbler_width / 2.0)
                    * corrected_angle)
                    .to_egui_pos2()
                    + pos_corrected.to_vec2(),
            ],
            egui::Stroke::new(dribbler_width, egui::Color32::GRAY),
        );
    }

    //TODO refactor plus joliment
    fn draw_field(&self, painter: &egui::Painter, offset: egui::Vec2, scale: f32) {
        let stroke: egui::Stroke = egui::Stroke::new(2.0 * scale, egui::Color32::WHITE);
        painter.rect_filled(
            egui::Rect::from_min_size(
                (egui::pos2(0.0, 0.0) * scale) + offset,
                egui::vec2(infos::FIELD_DEPTH, infos::FIELD_WIDTH) * scale,
            ),
            0.0,
            egui::Color32::from_rgb(0, 128, 0),
        );
        painter.hline(
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.x
                ..=((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.x,
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.y,
            stroke,
        );
        painter.hline(
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.x
                ..=((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.x,
            ((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.y,
            stroke,
        );
        painter.vline(
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.x,
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.y
                ..=((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.y,
            stroke,
        );
        painter.vline(
            ((infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.x,
            (infos::SPACE_BEFORE_LINE_SIDE * scale) + offset.y
                ..=((infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE) * scale) + offset.y,
            stroke,
        );

        let radius: f32 = infos::ENBUT_RADIUS * scale;

        // left enbut
        painter.rect_stroke(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x + (infos::SPACE_BEFORE_LINE_SIDE * scale),
                    offset.y + (((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) / 2.0) * scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH, infos::ENBUT_WIDTH) * scale,
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
                        + ((infos::FIELD_DEPTH
                            - infos::ENBUT_DEPTH
                            - infos::SPACE_BEFORE_LINE_SIDE)
                            * scale),
                    offset.y + (((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) / 2.0) * scale),
                ),
                egui::vec2(infos::ENBUT_DEPTH, infos::ENBUT_WIDTH) * scale,
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
            (infos::SPACE_BEFORE_LINE_SIDE * scale) - stroke.width / 2.0,
            infos::GOAL_WIDTH * scale,
        );

        // yellow goal
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(
                    offset.x,
                    offset.y + ((infos::FIELD_WIDTH - infos::GOAL_WIDTH) / 2.0) * scale,
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
                    offset.x + (infos::FIELD_DEPTH * scale) - goal_size.x,
                    offset.y + ((infos::FIELD_WIDTH - infos::GOAL_WIDTH) / 2.0) * scale,
                ),
                goal_size,
            ),
            0.0,
            egui::Color32::BLUE,
        );
    }
}
