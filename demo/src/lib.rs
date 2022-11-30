use std::ffi::c_void;

use egui_demo_lib::DemoWindows;

use uegui::{EGuiInitializer, UnityContext};

#[no_mangle]
pub extern "C" fn init(initializer: uegui::UnityInitializer) -> EGuiInitializer {
    let context = UnityContext::new(initializer, |cc| {
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "unity".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/SimSun.ttc")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "unity".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("unity".to_owned());
        cc.set_fonts(fonts);
        MyApp::default()
    });
    EGuiInitializer {
        update: update as _,
        app: Box::leak(Box::new(context)) as *mut UnityContext<MyApp> as _,
    }
}

//TODO use macro to replace this
#[no_mangle]
extern "C" fn update(input: uegui::Buffer, data: *mut c_void, destroy: u32) {
    unsafe {
        let app = data as *mut UnityContext<MyApp>;
        if destroy != 0 {
            let _ = Box::from_raw(app);
        } else {
            let app: &mut UnityContext<MyApp> = &mut *app;
            if let Err(err) = app.update(input) {
                eprint!("unexpected error:{:?}", err);
            }
        }
    }
}

#[derive(Default)]
struct MyApp {
    demo: DemoWindows,
}

impl uegui::App for MyApp {
    fn update(&mut self, ctx: &egui::Context) {
        self.demo.ui(ctx);
    }
}