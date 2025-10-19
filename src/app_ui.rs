use std::collections::HashMap;
use std::sync::Arc;

use nalgebra::vector;
use rerun::{AsComponents, Boxes2D, LineStrip2D, LineStrips2D, RecordingStream};
use rerun::{Color, DynamicArchetype, EntityPath, Points2D, Radius, TextLog};

use crate::{
    infos, robot::RobotHandler, simulator::SimulatorApp, vector_converter::EguiConvertCompatibility,
};

const BUTTON_PANEL_WIDTH: f32 = 150.0;

pub trait AppContainer: Default {
    fn start(self);
}

pub struct RerunContainer {
    pub simulation: SimulatorApp,
    pub rec: RecordingStream,
    pub robot_handle_to_color: HashMap<RobotHandler, Color>,
}

impl Default for RerunContainer {
    fn default() -> Self {
        let simulation = SimulatorApp::default();
        let mut robot_handle_to_color = HashMap::new();
        robot_handle_to_color.insert(simulation.robots[0], Color::from_rgb(0, 0, 255));
        robot_handle_to_color.insert(simulation.robots[1], Color::from_rgb(255, 255, 255));
        robot_handle_to_color.insert(simulation.robots[2], Color::from_rgb(255, 0, 0));
        robot_handle_to_color.insert(simulation.robots[3], Color::from_rgb(0, 255, 0));
        let rec = rerun::RecordingStreamBuilder::new("simulator")
            .spawn()
            .unwrap();
        RerunContainer {
            simulation,
            rec,
            robot_handle_to_color,
        }
    }
}

impl AppContainer for RerunContainer {
    fn start(mut self) {
        self.rec
            .log("simulator_messages", &TextLog::new("hello"))
            .unwrap();
        self.simulation.rigid_body_set[self.simulation.ball_rigid_body_handle]
            .apply_impulse(vector![-100.0, 0.0], true);

        self.draw_field();
        loop {
            self.simulation.tick();
            // draw ball
            let ball_position = self.simulation.position_of_ball();
            self.rec
                .log(
                    "ball",
                    &Points2D::new([[ball_position.x, ball_position.y]])
                        .with_colors([Color::from_rgb(255, 128, 0)])
                        .with_radii([Radius::new_scene_units(infos::BALL_RADIUS)]),
                )
                .unwrap();
            for robot_handle in self.simulation.robots {
                self.draw_robot(&robot_handle);
            }
        }
    }
}

impl RerunContainer {
    fn draw_robot(&self, robot_handle: &RobotHandler) {
        let robot_position = self.simulation.position_of(&robot_handle);
        let robot_position = [robot_position.x, robot_position.y];
        self.rec
            .log(
                format!("Robot_{robot_handle}/structure"),
                &Points2D::new([robot_position])
                    .with_colors([self.robot_handle_to_color[&robot_handle]])
                    .with_radii([Radius::new_scene_units(infos::ROBOT_RADIUS)]),
            )
            .unwrap();

        // dribbler
        let robot_angle = *self.simulation.rotation_of(&robot_handle);
        let dribbler_length = infos::ROBOT_RADIUS * 60.0 / 100.0;
        let dribbler_width = infos::ROBOT_RADIUS * 20.0 / 100.0;

        let p1 = nalgebra::Complex::new(
            -dribbler_length,
            -infos::ROBOT_RADIUS + dribbler_width / 2.0,
        ) * robot_angle;
        let p1 = [p1.re + robot_position[0], p1.im + robot_position[1]];

        let p2 =
            nalgebra::Complex::new(dribbler_length, -infos::ROBOT_RADIUS + dribbler_width / 2.0)
                * robot_angle;
        let p2 = [p2.re + robot_position[0], p2.im + robot_position[1]];

        self.rec
            .log(
                format!("Robot_{robot_handle}/dribbler"),
                &LineStrips2D::new([[p1, p2]])
                    .with_radii([Radius::new_scene_units(dribbler_width)])
                    .with_draw_order(60.0),
            )
            .unwrap();
    }

    fn draw_field(&self) {
        // Field rect (filled green)
        let field_rect =
            Boxes2D::from_mins_and_sizes([[0.0, 0.0]], [[infos::FIELD_DEPTH, infos::FIELD_WIDTH]]).with_colors([Color::from_rgb(0, 255, 0)]);
        self.rec.log(
            "field",
            &field_rect
        ).unwrap();

        // Inner boundary lines (white)
        let top_left = [infos::SPACE_BEFORE_LINE_SIDE, infos::SPACE_BEFORE_LINE_SIDE];
        let top_right = [
            (infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE),
            infos::SPACE_BEFORE_LINE_SIDE,
        ];
        let bot_left = [
            infos::SPACE_BEFORE_LINE_SIDE,
            (infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE),
        ];
        let bot_right = [
            (infos::FIELD_DEPTH - infos::SPACE_BEFORE_LINE_SIDE),
            (infos::FIELD_WIDTH - infos::SPACE_BEFORE_LINE_SIDE),
        ];

        self.rec.log(
            "field/boundaries",
            &[LineStrips2D::new([[top_left, top_right, bot_right, bot_left]])],
        ).unwrap();

        // // Left enbut (rounded rectangle outline) - positioned just inside left inner line
        // let left_en_x = infos::SPACE_BEFORE_LINE_SIDE;
        // let left_en_y = ((infos::FIELD_WIDTH - infos::ENBUT_WIDTH) / 2.0);
        // let left_infos::ENBUT_RADIUSect = Rect2D::from_min_size(
        //     [left_en_x, left_en_y],
        //     [infos::ENBUT_DEPTH, infos::ENBUT_WIDTH],
        // );
        // self.rec.log_rounded_rect_outline(
        //     &format!("{}/enbut/left", entity_path),
        //     &left_infos::ENBUT_RADIUSect,
        //     infos::ENBUT_RADIUS,
        //     stroke_w,
        //     [1.0, 1.0, 1.0, 1.0],
        //     // rounding mask: emulate only right corners rounded by giving corner radii individually if API supports it.
        // );

        // // Right enbut (rounded rectangle outline)
        // let right_en_x = (infos::FIELD_DEPTH - infos::ENBUT_DEPTH - infos::SPACE_BEFORE_LINE_SIDE);
        // let right_en_y = left_en_y; // vertically centered same as left
        // let right_infos::ENBUT_RADIUSect = Rect2D::from_min_size(
        //     [right_en_x, right_en_y],
        //     [infos::ENBUT_DEPTH, infos::ENBUT_WIDTH],
        // );
        // self.rec.log_rounded_rect_outline(
        //     &format!("{}/enbut/right", entity_path),
        //     &right_infos::ENBUT_RADIUSect,
        //     infos::ENBUT_RADIUS,
        //     2.0,
        //     [1.0, 1.0, 1.0, 1.0],
        // );

        // // Goal sizes: x thickness = (SPACE_BEFORE_LINE_SIDE * scale) - stroke.width/2
        // let goal_thickness = (infos::SPACE_BEFORE_LINE_SIDE * scale) - stroke_w / 2.0;
        // let goal_size = [goal_thickness, infos::GOAL_WIDTH];

        // // Left goal (yellow) at field left edge, vertically centered
        // let left_goal_pos = [ox, ((infos::FIELD_WIDTH - infos::GOAL_WIDTH) / 2.0)];
        // let left_goal_rect = Rect2D::from_min_size(left_goal_pos, goal_size);
        // self.rec.log_rect(
        //     &format!("{}/goal/left", entity_path),
        //     &left_goal_rect,
        //     comp::Mesh::SolidColor([1.0, 1.0, 0.0, 1.0]), // yellow
        // );

        // // Right goal (blue)
        // let right_goal_pos = [
        //     infos::FIELD_DEPTH - goal_thickness,
        //     ((infos::FIELD_WIDTH - infos::GOAL_WIDTH) / 2.0),
        // ];
        // let right_goal_rect = Rect2D::from_min_size(right_goal_pos, goal_size);
        // self.rec.log_rect(
        //     &format!("{}/goal/right", entity_path),
        //     &right_goal_rect,
        //     comp::Mesh::SolidColor([0.0, 0.0, 1.0, 1.0]), // blue
        // );
    }
}

pub struct NoUIContainer {
    pub simulation: SimulatorApp,
}

impl AppContainer for NoUIContainer {
    fn start(mut self) {
        loop {
            self.simulation.tick();
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
        let _ = eframe::run_native("Simulator", options, Box::new(|_cc| Ok(Box::new(self))));
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
        self.simulation.tick();

        // Building UI :
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_size = ui.max_rect();
            ui.horizontal(|ui| {
                ui.set_height(window_size.height());
                //buttons panel
                ui.vertical(|ui| {
                    ui.set_width(BUTTON_PANEL_WIDTH);

                    ui.add(egui::Label::new(
                        egui::RichText::new(format!(
                            "{} : {}",
                            self.simulation.game_referee.score_team_left,
                            self.simulation.game_referee.score_team_right
                        ))
                        .size(60.0),
                    ));

                    ui.add_space(10.0);

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
