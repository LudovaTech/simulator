use app_ui::AppContainer;

mod simulator;
mod app_ui;
mod infos;
mod robot;
mod vector_converter;

fn main() {
    let app_container = app_ui::NoUIContainer::default();
    app_container.start();
}