use std::collections::HashMap;
use std::sync::Arc;

use nalgebra::vector;
use rerun::external::egui::{Color32, RichText};
use rerun::external::re_log_types::BlueprintActivationCommand;
use rerun::external::re_types::blueprint;
use rerun::external::re_viewer::App;
use rerun::external::re_viewer_context::TimeControl;
use rerun::{
    ArchetypeName, AsComponents, Boxes2D, ComponentDescriptor, LineStrip2D, LineStrips2D,
    RecordingStream, StoreId,
};
use rerun::{Color, DynamicArchetype, EntityPath, Points2D, Radius, TextLog};

use rerun::external::{
    arrow, eframe, egui, re_chunk_store, re_crash_handler, re_entity_db, re_grpc_server, re_log,
    re_log_types, re_memory, re_types, re_viewer, tokio,
};

use crate::player_action::{PlayerAction, PlayerActionPython};
use crate::{
    infos, robot::RobotHandler, simulator::Simulator, vector_converter::EguiConvertCompatibility,
};

const BUTTON_PANEL_WIDTH: f32 = 150.0;

const APP_ID: &str = "simulator_app";

#[derive(Debug, PartialEq)]
pub enum AppState {
    Configuration,
    Running,
    ReRunning,
}

pub struct RerunContainer {
    pub rerun_app: re_viewer::App,
    pub simulation: Simulator,
    pub rec: RecordingStream,
    pub robot_handle_to_color: HashMap<RobotHandler, Color>,
    pub app_state: AppState,
}

impl eframe::App for RerunContainer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.rerun_app.save(storage);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.app_state {
            AppState::Configuration => {
                // We are still in configuration mode
                self.configuration_ui(ctx);
            }
            AppState::Running | AppState::ReRunning => {
                if self.app_state == AppState::Running {
                    // We don't need physic if we are in ReRunning mode
                    self.tick();
                }
                // First add our panel(s):
                egui::SidePanel::left("my_side_panel")
                    .default_width(300.0)
                    .show(ctx, |ui| {
                        self.ui(ctx, ui);
                    });

                // Now show the Rerun Viewer in the remaining space:
                self.rerun_app.update(ctx, frame);
            }
        }
    }
}

impl RerunContainer {
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let main_thread_token = re_viewer::MainThreadToken::i_promise_i_am_on_the_main_thread();

        // Direct calls using the `log` crate to stderr. Control with `RUST_LOG=debug` etc.
        re_log::setup_logging();

        // Install handlers for panics and crashes that prints to stderr and send
        // them to Rerun analytics (if the `analytics` feature is on in `Cargo.toml`).
        re_crash_handler::install_crash_handlers(re_viewer::build_info());

        // Listen for gRPC connections from Rerun's logging SDKs.
        // There are other ways of "feeding" the viewer though - all you need is a `re_smart_channel::Receiver`.
        let (rx, _) = re_grpc_server::spawn_with_recv(
            "0.0.0.0:9876".parse()?,
            Default::default(),
            re_grpc_server::shutdown::never(),
        );

        let mut native_options = re_viewer::native::eframe_options(None);
        native_options.viewport = native_options.viewport.with_app_id(APP_ID);

        let startup_options = re_viewer::StartupOptions::default();

        // This is used for analytics, if the `analytics` feature is on in `Cargo.toml`
        let app_env = re_viewer::AppEnvironment::Custom("My Wrapper".to_owned());

        let window_title = "Simulator";
        eframe::run_native(
            window_title,
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

                // TODO : maybe it is not a good idea to initiate with values, switch to enum with states according to AppState
                // blocking : custom viewport...
                let simulation = Simulator::default();
                let mut robot_handle_to_color = HashMap::new();
                robot_handle_to_color
                    .insert(simulation.robots[0].clone(), Color::from_rgb(0, 0, 255));
                robot_handle_to_color
                    .insert(simulation.robots[1].clone(), Color::from_rgb(255, 255, 255));
                robot_handle_to_color
                    .insert(simulation.robots[2].clone(), Color::from_rgb(255, 0, 0));
                robot_handle_to_color
                    .insert(simulation.robots[3].clone(), Color::from_rgb(0, 255, 0));
                // We mix server and client
                let rec = rerun::RecordingStreamBuilder::new("simulator")
                    .spawn()
                    .unwrap();
                rerun_app.add_log_receiver(rx);

                let mut rerun_container = RerunContainer {
                    rerun_app,
                    simulation,
                    rec,
                    robot_handle_to_color,
                    app_state: AppState::Configuration,
                };
                rerun_container.init();
                Ok(Box::new(rerun_container))
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

// Panel ui
impl RerunContainer {
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
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

            if ui.button("Move Robot A1 Right").clicked() {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new("A", 1)]]
                    .add_force(vector![100.0, 0.0], true);
            }
            if ui.button("Move Robot A1 Left").clicked() {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new("A", 1)]]
                    .add_force(vector![-100.0, 0.0], true);
            }
            if ui.button("Move Robot A1 Up").clicked() {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new("A", 1)]]
                    .apply_impulse(vector![0.0, -100.0], true);
            }
            if ui.button("Move Robot A1 Down").clicked() {
                self.simulation.rigid_body_set
                    [self.simulation.robot_to_rigid_body_handle[&RobotHandler::new("A", 1)]]
                    .apply_impulse(vector![0.0, 100.0], true);
            }
        });
        ui.separator();
        if let Some(entity_database) = self.rerun_app.recording_db() {
            // let query =
            //     re_chunk_store::LatestAtQuery::new(timeline, at)
            //re_chunk_store::LatestAtQuery::latest(re_log_types::TimelineName::log_time());
            // Print Component Descriptors
            // println!("{:?}", entity_database.storage_engine().store().all_components_for_entity(&EntityPath::from_file_path(std::path::Path::new("/ball"))));
            // Some({ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:positions", component_type: Some("rerun.components.Position2D") },
            // ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:colors", component_type: Some("rerun.components.Color") },
            // ComponentDescriptor { archetype: Some("rerun.archetypes.Points2D"), component: "Points2D:radii", component_type: Some("rerun.components.Radius") }})
            //let time = RecordingStream::now(&self)
            if let Some(blueprint_ctx) = self.rerun_app.blueprint_ctx(&StoreId::new(
                rerun::StoreKind::Blueprint,
                APP_ID,
                self.rec.store_info().unwrap().recording_id().to_string(),
            )) {
                let time_ctrl = TimeControl::from_blueprint(&blueprint_ctx);
                println!("time : {:?}", time_ctrl.time());
            } else {
                println!("notime");
            }
            
            let component_pos = ComponentDescriptor {
                archetype: Some("rerun.archetypes.Points2D".into()),
                component: "Points2D:positions".into(),
                component_type: Some("rerun.components.Position2D".into()),
            };
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
        }
    }

    fn configuration_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bienvenue sur le simulateur !");
            ui.label("Sélectionnez l'emplacement du code source des deux équipes");
            ui.label(RichText::new("Attention! le code sélectionné sera executé sur votre machine. N'entrez que du code auquel vous faites confiance.").color(Color32::RED));
            ui.add_space(20.0);
            let mut new_states = Vec::new();
            for (n, player_action) in self.simulation.player_action.iter_mut().enumerate() {
                match player_action {
                    PlayerAction::Invalid{ path, err_message } => {
                        ui.heading(format!("Equipe {} :", n+1));
                        let response = ui.text_edit_singleline( path);
                        if response.changed() {
                            new_states.push((n, PlayerAction::validate_path(&path)));
                        }
                        if let Some(err_message) = err_message {
                            ui.label(RichText::new(format!("{err_message}")).color(Color32::ORANGE));
                        }
                    },
                    PlayerAction::Python(PlayerActionPython { name, path, ..}) => {
                        ui.heading(format!("Equipe {name} :"));
                        ui.label(format!("code source : {}", path));
                        if ui.button(format!("enlever {}", name)).clicked() {
                            new_states.push((n, PlayerAction::default()));
                        }
                    }
                }
            }
            // Apply new states
            for (n, new_state) in new_states {
                self.simulation.player_action[n] = new_state;
            }

            ui.separator();
            if self.simulation.are_all_teams_ready() {
                if ui.button("Lancer la simulation !").clicked() {
                    self.app_state = AppState::Running;
                }
            } else {
                if ui.button("Démarrer une session sans équipes").clicked() {
                    // Let's remove all the configuration that the user made
                    self.simulation.player_action = Default::default();
                    self.app_state = AppState::ReRunning;
                }
                ui.label("cela permet par exemple d'importer le fichier d'un match enregistré pour le revoir.");
            }
        });
    }
}

// Simulation calls
impl RerunContainer {
    fn init(&mut self) {
        self.simulation.rigid_body_set[self.simulation.ball_rigid_body_handle]
            .apply_impulse(vector![-100.0, 0.0], true);
        self.draw_field();
    }

    fn tick(&mut self) {
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

        // We accept the performance cost of clone to avoid putting lifetimes everywhere
        for robot_handle in self.simulation.robots.clone() {
            self.draw_robot(&robot_handle);
        }
    }
}

// Draw utilities
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
            Boxes2D::from_mins_and_sizes([[0.0, 0.0]], [[infos::FIELD_DEPTH, infos::FIELD_WIDTH]])
                .with_colors([Color::from_rgb(0, 255, 0)]);
        self.rec.log_static("field", &field_rect).unwrap();

        self.rec
            .log_static(
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

pub struct NoUIContainer {
    pub simulation: Simulator,
}

impl NoUIContainer {
    pub fn start(mut self) {
        loop {
            self.simulation.tick();
        }
    }
}

impl Default for NoUIContainer {
    fn default() -> Self {
        NoUIContainer {
            simulation: Simulator::default(),
        }
    }
}
