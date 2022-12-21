use egui_demo_lib::DemoWindows;

uegui::init!(MyDemoApp, |cc| {
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
    MyDemoApp::default()
});

#[derive(Default)]
struct MyDemoApp {
    demo: DemoWindows,
}

impl uegui::App for MyDemoApp {
    fn update(&mut self, ctx: &egui::Context) {
        log::info!("update now, input:{}", ctx.wants_keyboard_input());
        self.demo.ui(ctx);
    }
}
