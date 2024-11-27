use app_ui::AppUI;

mod app;
mod app_ui;

// fn amain() -> Result<(), eframe::Error> {
//     gui::start()
// }

fn main() {
    let app_container = app_ui::AppUI::default();
    
    // let world = Box::new(world::World::default());
    // let reference: &'static mut world::World = Box::leak(world);
    // gui::start(reference).expect("Erreur lors du lancement du gui");
}