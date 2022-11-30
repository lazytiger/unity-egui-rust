//! Bridge from unity and egui.
//! Egui must be embeded in unity, so initialization should be invoked in unity.
//! Bridge provide an `init` function for unity to do initialization works.
//! Unity should provide something functionality for egui to gather input and paint meshes.
//! On the other side, egui should provide a function to be called in every frame.
//! All these works be done in `init` function.

use egui::{ClippedPrimitive, Context, ImageData, TextureFilter, TextureId};
use egui::epaint::{ImageDelta, Primitive};

use crate::{App, Buffer};
use crate::input::parse_input;

/// Unity provided functions for painting.
/// `set_texture` add or update texture in unity.
/// `rem_texture` remove texture in unity.
/// `begin_paint` called before paint begin, clear data for last frame.
/// `paint_mesh` generate and paint mesh in unity.
/// `end_paint` do something after paint in unity.
#[repr(C)]
pub struct UnityInitializer {
    /// set_texture(id, offsetX, offsetY, width, height, filter_mode, data)
    set_texture: extern "C" fn(u64, u32, u32, u32, u32, u32, *const u8),
    /// rem_texture(id)
    rem_texture: extern "C" fn(u64),
    /// begin_paint()
    begin_paint: extern "C" fn(),
    /// paint_mesh(texture_id, vertex_count, vertex_buffer, index_count, index_buffer, bound_min_x, bound_min_y, bound_max_x, bound_max_y)
    paint_mesh: extern "C" fn(u64, u32, *const u8, u32, *const u8, f32, f32, f32, f32),
    /// end_paint()
    end_paint: extern "C" fn(),
}

/// Context used by unity.
pub struct UnityContext<T: App> {
    context: Context,
    unity: UnityInitializer,
    app: T,
}

impl<T: App> UnityContext<T> {
    pub fn new<C: FnOnce(&Context) -> T>(initializer: UnityInitializer, creator: C) -> Self {
        let context = Context::default();
        let app = creator(&context);
        Self {
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

    /// Wrapper function for `set_texture` from unity.
    pub fn set_texture(&self, id: TextureId, image: ImageDelta) {
        let id = match id {
            TextureId::Managed(id) => id << 1,
            TextureId::User(id) => id << 1 + 1,
        };
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
                font.srgba_pixels(None).collect(),
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
        let id = match id {
            TextureId::Managed(id) => id << 1,
            TextureId::User(id) => id << 1 + 1,
        };
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
                let id = match mesh.texture_id {
                    TextureId::Managed(id) => id << 1,
                    TextureId::User(id) => id << 1 + 1,
                };
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
}