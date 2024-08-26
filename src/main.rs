mod gui;
mod infos;
mod vector_improver;
mod objects;
mod world;

use rapier2d::prelude::*;

// fn amain() -> Result<(), eframe::Error> {
//     gui::start()
// }

fn main() {
    let world = Box::new(world::World::default());
    let reference: &'static mut world::World = Box::leak(world);
    gui::start(reference).expect("Erreur lors du lancement du gui");
}