use eframe::App;

use crate::app::SimulatorApp;

pub trait AppContainer: Default {
    fn update_frame(&mut self);
}

pub struct AppUI {
    pub app: SimulatorApp,
}

impl AppContainer for AppUI {
    fn update_frame(&mut self) {
        self.app.update();
        self.update(ctx, frame);
    }
}

impl Default for AppUI {
    fn default() -> Self {
        AppUI {
            app: SimulatorApp {},
        }
    }
}

impl eframe::App for AppUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}
}

pub struct NoUI {
    pub app: SimulatorApp,
}

impl AppContainer for NoUI {
    fn update_frame(&mut self) {
        self.app.update();
    }
}

impl Default for NoUI {
    fn default() -> Self {
        NoUI {
            app: SimulatorApp {},
        }
    }
}
