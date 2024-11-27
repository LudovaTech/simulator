use app_ui::AppContainer;

mod simulator;
mod app_ui;
mod infos;

fn main() {
    let app_container = app_ui::AppUIContainer::default();
    app_container.start();
}