pub use egui;

pub use bridge::init;
pub use bridge::UnityInitializer;

mod proto;
mod bridge;
mod input;

#[repr(C)]
pub struct Buffer {
    pub data: *const u8,
    pub len: usize,
}

pub trait App: Send + Sync {
    fn update(&mut self, context: &egui::Context);
}

pub type AppCreator = Box<dyn FnOnce(&egui::Context) -> Box<dyn App>>;
