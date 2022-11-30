//! This is a library for interop between unity and egui.
//! When writing your own application you should create a new cargo library project with crate-type
//! has cdylib. Then an c type function named init should be provided as the following code
//! ```
//! #[no_mangle]
//! pub extern "C" fn init(initializer: uegui::UnityInitializer) -> *const c_void {
//!     uegui::init(initializer, Box::new(|cc| {
//!        Box::new(MyApp::default())
//!     }))
//! }
//!
//! struct MyApp {
//!     name: String,
//!     age: u32,
//! }
//!
//! impl Default for MyApp {
//!     fn default() -> Self {
//!         Self {
//!             name: "Arthur".to_owned(),
//!             age: 42,
//!         }
//!     }
//! }
//!
//! impl uegui::App for MyApp {
//!     fn update(&mut self, ctx: &egui::Context) {
//!         egui::CentralPanel::default().show(ctx, |ui| {
//!             ui.heading("My egui Application");
//!             ui.horizontal(|ui| {
//!                 ui.label("Your name: ");
//!                 ui.text_edit_singleline(&mut self.name);
//!             });
//!             ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
//!             if ui.button("Click each year").clicked() {
//!                 self.age += 1;
//!             }
//!             ui.label(format!("Hello '{}', age {}", self.name, self.age));
//!         });
//!     }
//! }
//! ```
//!
pub use egui;

pub use bridge::init;
pub use bridge::UnityInitializer;

mod bridge;
mod input;
mod proto;

/// Wrapper struct used to interchange binary data from c# to rust.
#[repr(C)]
pub struct Buffer {
    pub data: *const u8,
    pub len: usize,
}

/// Application trait like eframe.
pub trait App: Send + Sync {
    fn update(&mut self, context: &egui::Context);
}

pub type AppCreator = Box<dyn FnOnce(&egui::Context) -> Box<dyn App>>;
