use std::collections::HashMap;
use std::fmt::Debug;

use nalgebra::vector;
use rerun::external::egui::{Color32, RichText};
use rerun::external::re_viewer::App;
use rerun::{
    Boxes2D, LineStrips2D,
    RecordingStream,
};
use rerun::{Color, Points2D, Radius};

use rerun::external::{
    arrow, eframe, egui, re_crash_handler, re_grpc_server, re_log, re_viewer,
};

use crate::player_action::{CodeValidationError, PlayerActionPython, PlayerCode, validate_path};
use crate::robot::RobotBuilder;
use crate::{
    infos, robot::RobotHandler, simulator::Simulator,
};

const BUTTON_PANEL_WIDTH: f32 = 150.0;

const APP_ID: &str = "simulator_app";

#[derive(Debug)]
pub enum AppState {
    Configuration(AppConfiguration),
    Running(AppRunning),
    ReRunning(AppReRunning),
}
use AppState::*;

impl Default for AppState {
    fn default() -> Self {
        Configuration(Default::default())
    }
}

impl AppState {
    pub fn init(&mut self, rec: &mut RecordingStream) {
        match self {
            Configuration(..) => panic!("should not be called"),
            Running(running) => running.init(rec),
            ReRunning(..) => {}
        }
    }
}

impl eframe::App for SimulatorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.rerun_app.save(storage);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let signal = match &mut self.state {
            Configuration(config) => {
                // We are still in configuration mode
                config.ui_config(&ctx, &mut self.rerun_app, &mut self.rec)
            }
            Running(running) => {
                running.tick(&mut self.rec);
                running.ui_running(&ctx, &mut self.rerun_app, &mut self.rec);
                // Show rerun app
                self.rerun_app.update(ctx, frame);
                None
            }
            ReRunning(re_running) => {
                re_running.ui_re_running(&ctx, &mut self.rerun_app, &mut self.rec);
                self.rerun_app.update(ctx, frame);
                None
            } // | AppState::ReRunning => {
              //     if self.app_state == AppState::Running {
              //         // We don't need physic if we are in ReRunning mode
              //         self.tick();
              //     }
              //     // First add our panel(s):
              //     egui::SidePanel::left("my_side_panel")
              //         .default_width(300.0)
              //         .show(ctx, |ui| {
              //             self.ui(ctx, ui);
              //         });

              //     // Now show the Rerun Viewer in the remaining space:
              //     self.rerun_app.update(ctx, frame);
              // }
        };
        if let Some(signal) = signal
            && let Configuration(conf) = &mut self.state
        {
            match signal {
                AppStateMutateSignal::ToRun => self.state = conf.run(),
                AppStateMutateSignal::ToReRun => self.state = conf.re_run(),
            }
            self.state.init(&mut self.rec);
        }
    }
}

#[derive(Debug, Default)]
pub struct AppConfiguration {
    pub team_config: [TeamConfigState; 2],
}

#[derive(Debug)]
pub enum TeamConfigState {
    Config {
        path: String,
        err_message: Option<CodeValidationError>,
    },
    Valid(PlayerCode),
}

impl Default for TeamConfigState {
    fn default() -> Self {
        TeamConfigState::Config {
            path: String::new(),
            err_message: None,
        }
    }
}

pub struct AppRunning {
    pub simulation: Simulator,
    pub robot_handle_to_color: HashMap<RobotHandler, Color>,
}

impl Debug for AppRunning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppRunning")
            //.field("simulator", &self.simulation)
            .finish_non_exhaustive()
    }
}

pub struct AppReRunning {
    pub rerun_app: re_viewer::App,
}

impl Debug for AppReRunning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppReRunning").finish_non_exhaustive()
    }
}

// pub struct RerunContainer {
//     pub rerun_app: re_viewer::App,
//     pub simulation: Simulator,
//     pub rec: RecordingStream,
//     pub robot_handle_to_color: HashMap<RobotHandler, Color>,
//     pub app_state: AppState,
// }

impl AppRunning {
    fn ui_running(
        &mut self,
        ctx: &egui::Context,
        rerun_app: &mut re_viewer::App,
        rec: &mut RecordingStream,
    ) {
        egui::SidePanel::left("SIMULATOR")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.vertical_centered(|ui| {
                    ui.strong("SIMULATOR");
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!(
                            "{} : {}",
                            self.simulation.game_referee.score_team_left,
                            self.simulation.game_referee.score_team_right
                        ))
                        .size(60.0),
                    ));

                    ui.add_space(10.0);
                    let first_team_name = self.simulation.player_code[0].name();
                    let first_robot = RobotHandler::new(first_team_name, 1);
                    if ui.button("Move Robot A1 Right").clicked() {
                        self.simulation.rigid_body_set
                            [self.simulation.robot_to_rigid_body_handle[&first_robot]]
                            .add_force(vector![100.0, 0.0], true);
                    }
                    if ui.button("Move Robot A1 Left").clicked() {
                        self.simulation.rigid_body_set
                            [self.simulation.robot_to_rigid_body_handle[&first_robot]]
                            .add_force(vector![-100.0, 0.0], true);
                    }
                    if ui.button("Move Robot A1 Up").clicked() {
                        self.simulation.rigid_body_set
                            [self.simulation.robot_to_rigid_body_handle[&first_robot]]
                            .apply_impulse(vector![0.0, -100.0], true);
                    }
                    if ui.button("Move Robot A1 Down").clicked() {
                        self.simulation.rigid_body_set
                            [self.simulation.robot_to_rigid_body_handle[&first_robot]]
                            .apply_impulse(vector![0.0, 100.0], true);
                    }
                });
                ui.separator();
                // if let Some(entity_database) = self.rerun_app.recording_db() {
                // let query =
                //     re_chunk_store::LatestAtQuery::new(timeline, at)
                //re_chunk_store::LatestAtQuery::latest(re_log_types::TimelineName::log_time());
                // Print Component Descriptors
                // println!("{:?}", entity_database.storage_engine().store().all_components_for_entity(&EntityPath::from_file_path(std::path::Path::new("/ball"))));
                // Some({ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:positions", component_type: Some("rerun.components.Position2D") },
                // ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:colors", component_type: Some("rerun.components.Color") },
                // ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:radii", component_type: Some("rerun.components.Radius") }})
                //let time = RecordingStream::now(&self)
                // if let Some(blueprint_ctx) = self.rerun_app.blueprint_ctx(&StoreId::new(
                //     rerun::StoreKind::Blueprint,
                //     APP_ID,
                //     self.rec.store_info().unwrap().recording_id().to_string(),
                // )) {
                //     let time_ctrl = TimeControl::from_blueprint(&blueprint_ctx);
                //     println!("time : {:?}", time_ctrl.time());
                // } else {
                //     println!("notime");
                // }

                // let component_pos = ComponentDescriptor {
                //     archetype: Some("rerun.archetypes.Points2D".into()),
                //     component: "Points2D:positions".into(),
                //     component_type: Some("rerun.components.Position2D".into()),
                // };
                // let results = entity_database.latest_at(&query, &EntityPath::from_file_path(std::path::Path::new("/ball")), [&component_pos]);
                // // println!("result : {:?}", result.get_required(&component_pos).unwrap());
                // if let Some(data) = results.component_batch_raw(&component_pos) {
                //     egui::ScrollArea::vertical()
                //         .auto_shrink([false, true])
                //         .show(ui, |ui| {
                //             // Iterate over all the instances (e.g. all the points in the point cloud):

                //             let num_instances = data.len();
                //             println!("{:?}", num_instances);
                //             for i in 0..num_instances {
                //                 ui.label(format_arrow(&*data.slice(i, 1)));
                //             }
                //         });
                // };
                // }
            });
    }
}

impl AppReRunning {
    pub fn ui_re_running(
        &mut self,
        ctx: &egui::Context,
        rerun_app: &mut re_viewer::App,
        rec: &mut RecordingStream,
    ) {
        // TODO
    }
}

#[derive(Debug)]
enum AppStateMutateSignal {
    ToRun,
    ToReRun,
}

impl AppConfiguration {
    fn ui_config(
        &mut self,
        ctx: &egui::Context,
        rerun_app: &mut re_viewer::App,
        rec: &mut RecordingStream,
    ) -> Option<AppStateMutateSignal> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bienvenue sur le simulateur !");
            ui.label("Sélectionnez l'emplacement du code source des deux équipes");
            ui.label(RichText::new("Attention! le code sélectionné sera executé sur votre machine. N'entrez que du code auquel vous faites confiance.").color(Color32::RED));
            ui.add_space(20.0);
            let mut new_states = Vec::new();
            for (n, team_config_state) in self.team_config.iter_mut().enumerate() {
                match team_config_state {
                    TeamConfigState::Config{ path, err_message } => {
                        ui.heading(format!("Equipe {} :", n+1));
                        let response = ui.text_edit_singleline( path);
                        if response.changed() {
                            let validation = validate_path(&path);
                            new_states.push((n, match validation {
                                Ok(valid) => TeamConfigState::Valid(valid),
                                Err(CodeValidationError::Empty) => TeamConfigState::Config { path: path.to_string(), err_message: None },
                                Err(err) => TeamConfigState::Config { path: path.to_string(), err_message: Some(err) },
                            }));
                        }
                        if let Some(err_message) = err_message {
                            ui.label(RichText::new(format!("{err_message}")).color(Color32::ORANGE));
                        }
                    },
                    TeamConfigState::Valid(PlayerCode::Python(PlayerActionPython { name, path, ..})) => {
                        ui.heading(format!("Equipe {name} :"));
                        ui.label(format!("code source : {}", path));
                        if ui.button(format!("enlever {}", name)).clicked() {
                            new_states.push((n, TeamConfigState::default()));
                        }
                    }
                }
            }
            // Apply new states
            for (n, new_state) in new_states {
                self.team_config[n] = new_state;
            }

            ui.separator();
            if self
                .team_config
                .iter()
                .all(|tcs| matches!(tcs, TeamConfigState::Valid(..))) {
                if ui.button("Lancer la simulation !").clicked() {
                    return Some(AppStateMutateSignal::ToRun);
                }
            } else {
                if ui.button("Démarrer une session sans équipes").clicked() {
                    return Some(AppStateMutateSignal::ToReRun);
                }
                ui.label("cela permet par exemple d'importer le fichier d'un match enregistré pour le revoir.");
            }
            None
        }).inner
    }

    /// Mutate the app to run mode
    fn run(&mut self) -> AppState {
        // mem::take replaces the value in self with its default. Usefull as TeamConfigState is not Copy
        let local_configs = std::mem::take(&mut self.team_config);
        let [
            TeamConfigState::Valid(mut team1),
            TeamConfigState::Valid(mut team2),
        ] = local_configs
        else {
            panic!("Cannot mutate to state run with config {:?}", self);
        };

        // ensure unique team name
        let mut name1 = team1.name().to_owned();
        let mut name2 = team2.name().to_owned();
        if name1 == name2 {
            name1 += "_1";
            name2 += "_2";
            team1._set_name(&name1);
            team2._set_name(&name2);
        }

        let simulation = Simulator::new(
            [
                RobotBuilder::from_basic_robot(&name1, 1, vector!(50.0, 50.0)),
                RobotBuilder::from_basic_robot(&name1, 2, vector!(50.0, 75.0)),
                RobotBuilder::from_basic_robot(&name2, 1, vector!(50.0, 100.0)),
                RobotBuilder::from_basic_robot(&name2, 2, vector!(50.0, 125.0)),
            ],
            [team1, team2],
        );
        let mut robot_handle_to_color = HashMap::new();
        robot_handle_to_color.insert(simulation.robots[0].clone(), Color::from_rgb(0, 0, 255));
        robot_handle_to_color.insert(simulation.robots[1].clone(), Color::from_rgb(255, 255, 255));
        robot_handle_to_color.insert(simulation.robots[2].clone(), Color::from_rgb(255, 0, 0));
        robot_handle_to_color.insert(simulation.robots[3].clone(), Color::from_rgb(0, 255, 0));

        Running(AppRunning {
            simulation,
            robot_handle_to_color,
        })
    }

    /// Mutate the app to rerun mode
    fn re_run(&mut self) -> AppState {
        todo!()
    }
}

pub struct SimulatorApp {
    pub state: AppState,
    pub rerun_app: re_viewer::App,
    pub rec: RecordingStream,
}

impl Debug for SimulatorApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulatorApp")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl SimulatorApp {
    pub fn new(rerun_app: App, rec: RecordingStream) -> Self {
        Self {
            state: AppState::default(),
            rerun_app,
            rec,
        }
    }

    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        // Direct calls using the `log` crate to stderr. Control with `RUST_LOG=debug` etc.
        re_log::setup_logging();
        // Install handlers for panics and crashes that prints to stderr and send
        // them to Rerun analytics (if the `analytics` feature is on in `Cargo.toml`).
        re_crash_handler::install_crash_handlers(re_viewer::build_info());
        let mut native_options = re_viewer::native::eframe_options(None);
        native_options.viewport = native_options.viewport.with_app_id(APP_ID);

        let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();

        // Listen for gRPC connections from Rerun's logging SDKs.
        // There are other ways of "feeding" the viewer though - all you need is a `re_smart_channel::Receiver`.
        let (rx, _) = re_grpc_server::spawn_with_recv(
            "0.0.0.0:9876".parse().unwrap(),
            Default::default(),
            re_grpc_server::shutdown::never(),
        );

        let mut native_options = re_viewer::native::eframe_options(None);
        native_options.viewport = native_options.viewport.with_app_id(APP_ID);

        let startup_options = re_viewer::StartupOptions {
            hide_welcome_screen: true,
            // panel_state_overrides: PanelStateOverrides { top: (), blueprint: (), selection: (), time: () },
            // on_event HERE
            ..Default::default()
        };

        // This is used for analytics, if the `analytics` feature is on in `Cargo.toml`
        let app_env = re_viewer::AppEnvironment::Custom("My Wrapper".to_owned());

        eframe::run_native(
            "Simulator",
            native_options,
            Box::new(move |cc| {
                re_viewer::customize_eframe_and_setup_renderer(cc)?;
                let mut rerun_app = re_viewer::App::new(
                    main_thread_token,
                    re_viewer::build_info(),
                    app_env,
                    startup_options,
                    cc,
                    None,
                    re_viewer::AsyncRuntimeHandle::from_current_tokio_runtime_or_wasmbindgen()?,
                );

                // We mix server and client
                let rec = rerun::RecordingStreamBuilder::new("simulator")
                    .spawn()
                    .unwrap();
                rerun_app.add_log_receiver(rx);
                Ok(Box::new(Self::new(rerun_app, rec)))
            }),
        )?;

        Ok(())
    }
}

fn format_arrow(array: &dyn arrow::array::Array) -> String {
    use arrow::util::display::{ArrayFormatter, FormatOptions};

    let num_bytes = array.get_buffer_memory_size();
    if array.len() == 1 {
        // && num_bytes < 256 // TODO why ?
        // Print small items:
        let options = FormatOptions::default();
        if let Ok(formatter) = ArrayFormatter::try_new(array, &options) {
            return formatter.value(0).to_string();
        }
    }

    // Fallback:
    format!("{num_bytes} bytes")
}

// Simulation calls
impl AppRunning {
    fn init(&mut self, rec: &mut RecordingStream) {
        self.simulation.rigid_body_set[self.simulation.ball_rigid_body_handle]
            .apply_impulse(vector![-100.0, 0.0], true);
        self.draw_field(rec);
    }

    fn tick(&mut self, rec: &mut RecordingStream) {
        self.simulation.tick();
        // draw ball
        let ball_position = self.simulation.position_of_ball();
        rec.log(
            "ball",
            &Points2D::new([[ball_position.x, ball_position.y]])
                .with_colors([Color::from_rgb(255, 128, 0)])
                .with_radii([Radius::new_scene_units(infos::BALL_RADIUS)]),
        )
        .unwrap();

        // We accept the performance cost of clone to avoid putting lifetimes everywhere
        for robot_handle in self.simulation.robots.clone() {
            self.draw_robot(rec, &robot_handle);
        }
    }
}

/// Draw utilities
impl AppRunning {
    fn draw_robot(&self, rec: &mut RecordingStream, robot_handle: &RobotHandler) {
        let robot_position = self.simulation.position_of(&robot_handle);
        let robot_position = [robot_position.x, robot_position.y];
        rec.log(
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

        rec.log(
            format!("Robot_{robot_handle}/dribbler"),
            &LineStrips2D::new([[p1, p2]])
                .with_radii([Radius::new_scene_units(dribbler_width)])
                .with_draw_order(60.0),
        )
        .unwrap();
    }

    fn draw_field(&self, rec: &mut RecordingStream) {
        // Field rect (filled green)
        let field_rect =
            Boxes2D::from_mins_and_sizes([[0.0, 0.0]], [[infos::FIELD_DEPTH, infos::FIELD_WIDTH]])
                .with_colors([Color::from_rgb(0, 255, 0)]);
        rec.log_static("field", &field_rect).unwrap();

        rec.log_static(
            "field/boundaries",
            &[Boxes2D::from_mins_and_sizes(
                [[infos::SPACE_BEFORE_LINE_SIDE, infos::SPACE_BEFORE_LINE_SIDE]],
                [[
                    infos::FIELD_DEPTH - 2.0 * infos::SPACE_BEFORE_LINE_SIDE,
                    infos::FIELD_WIDTH - 2.0 * infos::SPACE_BEFORE_LINE_SIDE,
                ]],
            )],
        )
        .unwrap();

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

// TODO
// pub struct NoUIContainer {
//     pub simulation: Simulator,
// }

// impl NoUIContainer {
//     pub fn start(mut self) {
//         loop {
//             self.simulation.tick();
//         }
//     }
// }

// impl Default for NoUIContainer {
//     fn default() -> Self {
//         NoUIContainer {
//             simulation: Simulator::default(),
//         }
//     }
// }
