use app_ui::AppContainer;

mod app_ui;
mod game_referee;
mod infos;
mod robot;
mod simulator;
mod vector_converter;

fn main() {
    let app_container = app_ui::RerunContainer::default();
    app_container.start();
}
