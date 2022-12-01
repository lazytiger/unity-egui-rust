use eframe::{App, Frame};
use egui_demo_lib::DemoWindows;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|_cc| Box::new(MyEguiApp::default())),
    );
}

#[derive(Default)]
struct MyEguiApp {
    demo: DemoWindows,
}

impl App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.demo.ui(ctx);
    }
}
