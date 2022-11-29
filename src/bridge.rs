use std::ffi::c_void;
use std::panic;
use std::sync::RwLock;

use egui::{ClippedPrimitive, Context, ImageData, TextureFilter, TextureId};
use egui::epaint::{ImageDelta, Primitive};
use lazy_static::lazy_static;

use crate::{App, AppCreator, Buffer};
use crate::input::parse_input;

#[repr(C)]
pub struct UnityInitializer {
    set_texture: extern "C" fn(u64, u32, u32, u32, u32, u32, *const u8),
    rem_texture: extern "C" fn(u64),
    begin_paint: extern "C" fn(),
    paint_mesh: extern "C" fn(u64, u32, *const u8, u32, *const u8, f32, f32, f32, f32),
    end_paint: extern "C" fn(),
}

lazy_static! {
    static ref INITIALIZER: RwLock<Option<UnityInitializer>> = RwLock::new(None);
    static ref APP:RwLock<Option<Box<dyn App>>> = RwLock::new(None);
    static ref CONTEXT:Context = Context::default();
}

pub fn init(initializer: UnityInitializer, app: AppCreator) -> *const c_void {
    INITIALIZER.write().unwrap().replace(initializer);
    APP.write().unwrap().replace(app(&CONTEXT));
    safe_update as _
}

pub fn safe_update(buffer: Buffer) -> u32 {
    let result = panic::catch_unwind(|| {
        update(buffer);
    });
    if let Err(err) = result {
        println!("unexpected error:{:?}", err);
        1
    } else {
        0
    }
}

fn update(buffer: Buffer) {
    let input = parse_input(buffer);
    CONTEXT.begin_frame(input);
    begin_paint();
    let mut app = APP.write().unwrap();
    app.as_mut().unwrap().update(&CONTEXT);
    let output = CONTEXT.end_frame();
    if output.repaint_after.is_zero() {
        return;
    }
    for id in output.textures_delta.free {
        rem_texture(id);
    }
    for (id, image) in output.textures_delta.set {
        set_texture(id, image);
    }
    let cps = CONTEXT.tessellate(output.shapes);
    for cp in cps {
        paint_mesh(cp);
    }
    end_paint();
}

pub fn set_texture(id: TextureId, image: ImageDelta) {
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
    let ui = INITIALIZER.read().unwrap();
    let ui = ui.as_ref().unwrap();
    (ui.set_texture)(
        id,
        offset_x,
        offset_y,
        width,
        height,
        filter_mode,
        data.as_ptr() as *const u8,
    )
}

pub fn rem_texture(id: TextureId) {
    let id = match id {
        TextureId::Managed(id) => id << 1,
        TextureId::User(id) => id << 1 + 1,
    };
    let ui = INITIALIZER.read().unwrap();
    let ui = ui.as_ref().unwrap();
    (ui.rem_texture)(id);
}

pub fn begin_paint() {
    let ui = INITIALIZER.read().unwrap();
    let ui = ui.as_ref().unwrap();
    (ui.begin_paint)()
}

pub fn paint_mesh(cp: ClippedPrimitive) {
    match cp.primitive {
        Primitive::Mesh(mesh) => {
            let id = match mesh.texture_id {
                TextureId::Managed(id) => id << 1,
                TextureId::User(id) => id << 1 + 1,
            };
            let ui = INITIALIZER.read().unwrap();
            let ui = ui.as_ref().unwrap();
            (ui.paint_mesh)(
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

pub fn end_paint() {
    let ui = INITIALIZER.read().unwrap();
    let ui = ui.as_ref().unwrap();
    (ui.end_paint)()
}
