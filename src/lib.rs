//! This is a library for interop between unity and egui.
//! When writing your own application you should create a new cargo library project with crate-type
//! has cdylib. Then use init macro as the following code
//! ```
//! uegui::init!(MyDemoApp, |cc| {
//!    MyApp::default()
//! });
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

/// Generate exported function used for unity.
/// ```
/// init!(MyApp, |_cc|{MyApp::default()});
/// ```
#[macro_export]
macro_rules! init {
    ($name:ident, $app:expr) => {
        #[no_mangle]
        pub extern "C" fn init(initializer: $crate::UnityInitializer) -> $crate::EGuiInitializer {
            let context = Box::new($crate::UnityContext::new(initializer, $app));
            context.init_log();
            $crate::EGuiInitializer {
                update: update as _,
                app: Box::leak(context) as *mut $crate::UnityContext<$name> as _,
            }
        }

        #[no_mangle]
        extern "C" fn update(input: $crate::Buffer, data: *mut std::ffi::c_void, destroy: u32) {
            if let Err(err) = std::panic::catch_unwind(|| unsafe {
                let app = data as *mut $crate::UnityContext<$name>;
                if destroy != 0 {
                    let _ = Box::from_raw(app);
                } else {
                    let app: &mut $crate::UnityContext<$name> = &mut *app;
                    if let Err(err) = app.update(input) {
                        log::error!("unexpected error:{:?}", err);
                    }
                }
            }) {
                log::error!("unwind error:{:?}", err);
            }
        }
    };
}
