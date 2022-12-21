//! Bridge from unity and egui.
//! Egui must be embeded in unity, so initialization should be invoked in unity.
//! Bridge provide an `init` function for unity to do initialization works.
//! Unity should provide something functionality for egui to gather input and paint meshes.
//! On the other side, egui should provide a function to be called in every frame.
//! All these works be done in `init` function.

use egui::epaint::{ImageDelta, Primitive};
use egui::output::OutputEvent;
use egui::{
    ClippedPrimitive, Context, ImageData, PlatformOutput, TextureFilter, TextureId, WidgetType,
};
use log::{set_logger, set_max_level, Level, LevelFilter, Metadata, Record};

use crate::input::parse_input;
use crate::{App, Buffer};

/// Unity provided functions for painting.
/// `set_texture` add or update texture in unity.
/// `rem_texture` remove texture in unity.
/// `begin_paint` called before paint begin, clear data for last frame.
/// `paint_mesh` generate and paint mesh in unity.
/// `end_paint` do something after paint in unity.
/// `show_keyboard` show ime in android.
#[repr(C)]
pub struct UnityInitializer {
    /// set_texture(id, offsetX, offsetY, width, height, filter_mode, data)
    set_texture: extern "system" fn(u64, u32, u32, u32, u32, u32, *const u8),
    /// rem_texture(id)
    rem_texture: extern "system" fn(u64),
    /// begin_paint()
    begin_paint: extern "system" fn(),
    /// paint_mesh(texture_id, vertex_count, vertex_buffer, index_count, index_buffer, bound_min_x, bound_min_y, bound_max_x, bound_max_y)
    paint_mesh: extern "system" fn(u64, u32, *const u8, u32, *const u8, f32, f32, f32, f32),
    /// end_paint()
    end_paint: extern "system" fn(),
    /// show_keyboard(show, string)
    show_keyboard: extern "system" fn(u32, *const u8, u32),
    /// show_log(show, string)
    show_log: extern "system" fn(i32, *const u8, i32),
}

pub struct UnityLogger {
    show_log: extern "system" fn(i32, *const u8, i32),
    log_level: LevelFilter,
}

/// Context used by unity.
pub struct UnityContext<T: App> {
    context: Context,
    unity: UnityInitializer,
    logger: UnityLogger,
    app: T,
    text: String,
}

fn texture_id_to_u64(id: TextureId) -> u64 {
    match id {
        TextureId::Managed(id) => id << 1,
        TextureId::User(id) => id << 1 + 1,
    }
}

impl<T: App> UnityContext<T> {
    pub fn new<C: FnOnce(&Context) -> T>(initializer: UnityInitializer, creator: C) -> Self {
        let context = Context::default();
        let app = creator(&context);
        Self {
            text: "".into(),
            logger: UnityLogger {
                show_log: initializer.show_log,
                log_level: LevelFilter::Trace,
            },
            unity: initializer,
            context,
            app,
        }
    }

    /// Update function called very frame from unity.
    /// 1. get input from unity
    /// 2. call `begin_frame` in egui
    /// 3. call `App::update` in egui
    /// 4. call `end_frame` in egui
    /// 5. return if not paint immediately
    /// 6. call `begin_paint` from unity
    /// 7. call `rem_texture` from unity
    /// 8. call `set_texture` from unity
    /// 9. call `paint_mesh` from unity
    /// 10. call `end_paint` from unity
    pub fn update(&mut self, buffer: Buffer) -> Result<(), protobuf::Error> {
        let input = parse_input(buffer)?;
        self.context.begin_frame(input);
        self.app.update(&self.context);
        let output = self.context.end_frame();
        if !output.repaint_after.is_zero() {
            return Ok(());
        }
        self.update_platform(&output.platform_output);
        self.show_keyboard(self.context.wants_keyboard_input());
        self.begin_paint();
        for id in output.textures_delta.free {
            self.rem_texture(id);
        }
        for (id, image) in output.textures_delta.set {
            self.set_texture(id, image);
        }
        let cps = self.context.tessellate(output.shapes);
        for cp in cps {
            self.paint_mesh(cp);
        }
        self.end_paint();
        Ok(())
    }

    pub fn update_platform(&mut self, platform: &PlatformOutput) {
        for e in &platform.events {
            let info = match e {
                OutputEvent::Clicked(info) => info,
                OutputEvent::DoubleClicked(info) => info,
                OutputEvent::FocusGained(info) => info,
                OutputEvent::TripleClicked(info) => info,
                OutputEvent::TextSelectionChanged(info) => info,
                OutputEvent::ValueChanged(info) => info,
            };
            match (info.typ, &info.current_text_value) {
                (WidgetType::TextEdit, Some(text)) => {
                    self.text = text.clone();
                }
                _ => (),
            }
        }
    }

    /// Wrapper function for `set_texture` from unity.
    pub fn set_texture(&self, id: TextureId, image: ImageDelta) {
        let id = texture_id_to_u64(id);
        let filter_mode = match image.options.minification {
            TextureFilter::Nearest => 1,
            TextureFilter::Linear => 2,
        };
        let (offset_x, offset_y) = match image.pos {
            Some(pos) => (pos[0] as u32, pos[1] as u32),
            _ => (0, 0),
        };
        let (width, height, data) = match image.image {
            ImageData::Color(color) => (color.size[0] as u32, color.size[1] as u32, color.pixels),
            ImageData::Font(font) => (
                font.size[0] as u32,
                font.size[1] as u32,
                font.srgba_pixels(Some(1.0)).collect(),
            ),
        };
        (self.unity.set_texture)(
            id,
            offset_x,
            offset_y,
            width,
            height,
            filter_mode,
            data.as_ptr() as *const u8,
        )
    }

    /// Wrapper function for `rem_texture` from unity.
    pub fn rem_texture(&self, id: TextureId) {
        let id = texture_id_to_u64(id);
        (self.unity.rem_texture)(id);
    }

    /// Wrapper function for `begin_paint` from unity.
    pub fn begin_paint(&self) {
        (self.unity.begin_paint)()
    }

    /// Wrapper function for `paint_mesh` from unity.
    pub fn paint_mesh(&self, cp: ClippedPrimitive) {
        match cp.primitive {
            Primitive::Mesh(mesh) => {
                let id = texture_id_to_u64(mesh.texture_id);
                (self.unity.paint_mesh)(
                    id,
                    mesh.vertices.len() as u32,
                    mesh.vertices.as_ptr() as *const u8,
                    mesh.indices.len() as u32,
                    mesh.indices.as_ptr() as *const u8,
                    cp.clip_rect.min.x,
                    cp.clip_rect.min.y,
                    cp.clip_rect.max.x,
                    cp.clip_rect.max.y,
                );
            }
            Primitive::Callback(_) => {
                unimplemented!("callback not supported");
            }
        }
    }

    /// Wrapper function for `end_paint` from unity.
    pub fn end_paint(&self) {
        (self.unity.end_paint)()
    }

    pub fn show_keyboard(&self, show: bool) {
        (self.unity.show_keyboard)(
            if show { 1 } else { 0 },
            self.text.as_ptr(),
            self.text.len() as u32,
        );
    }

    pub fn set_log_level(&mut self, level: LevelFilter) {
        self.logger.log_level = level;
    }

    pub fn init_log(&self) {
        let logger: &'static UnityLogger = unsafe { std::mem::transmute(&self.logger) };
        set_logger(logger)
            .map(|_| set_max_level(LevelFilter::Trace))
            .unwrap();
    }
}

impl log::Log for UnityLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.log_level >= metadata.level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = format!(
            "{}[{}:{}][{}]{}",
            chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.6f]"),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.level(),
            record.args(),
        );
        (self.show_log)(
            log_level_to_unity(record.level()),
            message.as_ptr(),
            message.len() as i32,
        );
    }

    fn flush(&self) {}
}

fn log_level_to_unity(level: Level) -> i32 {
    match level {
        Level::Error => 1,
        Level::Debug => 4,
        Level::Trace => 4,
        Level::Info => 4,
        Level::Warn => 3,
    }
}
