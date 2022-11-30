//! This is a library for interop between unity and egui.
//! When writing your own application you should create a new cargo library project with crate-type
//! has cdylib. Then an c type function named init should be provided as the following code
//! ```
//! #[no_mangle]
//! pub extern "C" fn init(initializer: uegui::UnityInitializer) -> *const c_void {
//!     let context = UnityContext::new(initializer, |cc| {
//!         MyApp::default()
//!     });
//!     EGuiInitializer {
//!         update: update as _,
//!         app: Box::leak(Box::new(context)) as *mut UnityContext<MyApp> as _,
//!     }
//! }
//!
//! //TODO use macro
//! #[no_mangle]
//! extern "C" fn update(input: uegui::Buffer, data: *mut c_void) {
//!     unsafe {
//!         let app: &mut UnityContext<MyApp> = &mut *(data as *mut UnityContext<MyApp>);
//!         if let Err(err) = app.update(input) {
//!             eprint!("unexpected error:{:?}", err);
//!         }
//!     }
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
use std::ffi::c_void;

pub use bridge::{UnityContext, UnityInitializer};

mod bridge;
mod input;
mod proto;

/// Wrapper struct used to interchange binary data from c# to rust.
#[repr(C)]
pub struct Buffer {
    pub data: *const u8,
    pub len: usize,
}

/// Wrapper struct for rust exported functions and data
#[repr(C)]
pub struct EGuiInitializer {
    /// update function pointer
    pub update: *const c_void,
    /// app data pointer
    pub app: *mut c_void,
}

/// Application trait like eframe.
pub trait App {
    fn update(&mut self, context: &egui::Context);
}

