mod app_ui;
mod game_referee;
mod infos;
mod player_action;
mod robot;
mod simulator;
mod vector_converter;

use rerun::external::{re_memory, tokio};

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

// fn no_container_main() {
//     let app_container = app_ui::NoUIContainer::default();
//     app_container.start();
// }

#[tokio::main]
async fn main() {
    app_ui::SimulatorApp::start().await.unwrap();
}
