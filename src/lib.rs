use std::default::Default;
use std::ptr::slice_from_raw_parts;
use std::sync::Mutex;

use egui::{CentralPanel, Context, ImageData, Key, RawInput, TextureFilter};
use egui::epaint::Primitive;
use egui::Event::PointerButton;
use lazy_static::lazy_static;
use protobuf::Message;

use crate::proto::common::{Pos2, Rect};
use crate::proto::input::{ButtonType, Event, EventType, Input, KeyType, Modifiers};
use crate::proto::output::{
    ClippedPrimitive, Color32, Mesh, Output, Texture, TextureId, Vertex,
};

mod proto;

lazy_static! {
    pub static ref CONTEXT: Context = Context::default();
    pub static ref BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}

#[repr(C)]
pub struct Buffer {
    pub data: *const u8,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn begin(buffer: Buffer) {
    let buffer = unsafe { &*slice_from_raw_parts(buffer.data, buffer.len) };
    let mut pb_input = Input::default();
    pb_input.merge_from_bytes(buffer).unwrap();
    let mut input = RawInput::default();
    input.screen_rect = pb_input.screen_rect.as_ref().map(rect_from_pb_to_native);
    input.has_focus = pb_input.has_focus;
    if pb_input.time > 0 {
        input.time = Some(pb_input.time as f64);
    }
    if pb_input.pixels_per_point > 0.0 {
        input.pixels_per_point = Some(pb_input.pixels_per_point);
    }
    if pb_input.max_texture_side > 0 {
        input.max_texture_side = Some(pb_input.max_texture_side as usize);
    }
    if pb_input.modifier.is_some() {
        input.modifiers = modifier_from_pb_to_native(pb_input.modifier.as_ref().unwrap());
    }
    input.predicted_dt = pb_input.predicted_dt;
    for event in pb_input.events {
        if let Some(event) = event_from_pb_to_native(event) {
            input.events.push(event);
        }
    }
    CONTEXT.begin_frame(input);
}

fn event_from_pb_to_native(e: Event) -> Option<egui::Event> {
    if e.et.enum_value().is_err() {
        return None;
    }
    match e.et.unwrap() {
        EventType::ET_NONE => None,
        EventType::COPY => Some(egui::Event::Copy),
        EventType::CUT => Some(egui::Event::Cut),
        EventType::PASTE => Some(egui::Event::Paste(e.paste)),
        EventType::TEXT => Some(egui::Event::Text(e.text)),
        EventType::KEY => {
            e
                .key
                .as_ref()
                .map(|e| {
                    e.key.enum_value().ok().map(|t| match t {
                        KeyType::KT_NONE => None,
                        KeyType::ArrowDown => Some(Key::ArrowDown),
                        KeyType::ArrowLeft => Some(Key::ArrowLeft),
                        KeyType::ArrowRight => Some(Key::ArrowRight),
                        KeyType::ArrowUp => Some(Key::ArrowUp),
                        KeyType::Escape => Some(Key::Escape),
                        KeyType::Tab => Some(Key::Tab),
                        KeyType::Backspace => Some(Key::Backspace),
                        KeyType::Enter => Some(Key::Enter),
                        KeyType::Space => Some(Key::Space),
                        KeyType::Insert => Some(Key::Insert),
                        KeyType::Delete => Some(Key::Delete),
                        KeyType::Home => Some(Key::Home),
                        KeyType::End => Some(Key::End),
                        KeyType::PageUp => Some(Key::PageUp),
                        KeyType::PageDown => Some(Key::PageDown),
                        KeyType::Num0 => Some(Key::Num0),
                        KeyType::Num1 => Some(Key::Num1),
                        KeyType::Num2 => Some(Key::Num2),
                        KeyType::Num3 => Some(Key::Num3),
                        KeyType::Num4 => Some(Key::Num4),
                        KeyType::Num5 => Some(Key::Num5),
                        KeyType::Num6 => Some(Key::Num6),
                        KeyType::Num7 => Some(Key::Num7),
                        KeyType::Num8 => Some(Key::Num8),
                        KeyType::Num9 => Some(Key::Num9),
                        KeyType::A => Some(Key::A),
                        KeyType::B => Some(Key::B),
                        KeyType::C => Some(Key::C),
                        KeyType::D => Some(Key::D),
                        KeyType::E => Some(Key::E),
                        KeyType::F => Some(Key::F),
                        KeyType::G => Some(Key::G),
                        KeyType::H => Some(Key::H),
                        KeyType::I => Some(Key::I),
                        KeyType::J => Some(Key::J),
                        KeyType::K => Some(Key::K),
                        KeyType::L => Some(Key::L),
                        KeyType::M => Some(Key::M),
                        KeyType::N => Some(Key::N),
                        KeyType::O => Some(Key::O),
                        KeyType::P => Some(Key::P),
                        KeyType::Q => Some(Key::Q),
                        KeyType::R => Some(Key::R),
                        KeyType::S => Some(Key::S),
                        KeyType::T => Some(Key::T),
                        KeyType::U => Some(Key::U),
                        KeyType::V => Some(Key::V),
                        KeyType::W => Some(Key::W),
                        KeyType::X => Some(Key::X),
                        KeyType::Y => Some(Key::Y),
                        KeyType::Z => Some(Key::Z),
                        KeyType::F1 => Some(Key::F1),
                        KeyType::F2 => Some(Key::F2),
                        KeyType::F3 => Some(Key::F3),
                        KeyType::F4 => Some(Key::F4),
                        KeyType::F5 => Some(Key::F5),
                        KeyType::F6 => Some(Key::F6),
                        KeyType::F7 => Some(Key::F7),
                        KeyType::F8 => Some(Key::F8),
                        KeyType::F9 => Some(Key::F9),
                        KeyType::F10 => Some(Key::F10),
                        KeyType::F11 => Some(Key::F11),
                        KeyType::F12 => Some(Key::F12),
                        KeyType::F13 => Some(Key::F13),
                        KeyType::F14 => Some(Key::F14),
                        KeyType::F15 => Some(Key::F15),
                        KeyType::F16 => Some(Key::F16),
                        KeyType::F17 => Some(Key::F17),
                        KeyType::F18 => Some(Key::F18),
                        KeyType::F19 => Some(Key::F19),
                        KeyType::F20 => Some(Key::F20),
                    })
                })
                .map(|kt| kt.unwrap_or_default())
                .unwrap_or_default().map(|kt| egui::Event::Key {
                key: kt,
                pressed: e.key.pressed,
                modifiers: modifier_from_pb_to_native(&e.key.modifiers),
            })
        }
        EventType::POINTER_MOVED => e
            .pointer_moved
            .as_ref()
            .map(pos2_from_pb_to_native)
            .map(egui::Event::PointerMoved),
        EventType::POINTER_BUTTON => e
            .pointer_button
            .as_ref()
            .map(|b| {
                b.button
                    .enum_value()
                    .ok()
                    .map(|bt| match bt {
                        ButtonType::BT_NONE => None,
                        ButtonType::PRIMARY => Some(egui::PointerButton::Primary),
                        ButtonType::SECONDARY => Some(egui::PointerButton::Secondary),
                        ButtonType::MIDDLE => Some(egui::PointerButton::Middle),
                        ButtonType::EXTRA1 => Some(egui::PointerButton::Extra1),
                        ButtonType::EXTRA2 => Some(egui::PointerButton::Extra2),
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .map(|bt| PointerButton {
                pos: pos2_from_pb_to_native(&e.pointer_button.pos),
                button: bt,
                pressed: e.pointer_button.pressed,
                modifiers: modifier_from_pb_to_native(&e.pointer_button.modifiers),
            }),
        EventType::POINTER_GONE => Some(egui::Event::PointerGone),
        EventType::SCROLL => e
            .scroll
            .as_ref()
            .map(pos2_from_pb_to_native)
            .map(|pos| egui::Event::Scroll(egui::Vec2 { x: pos.x, y: pos.y })),
        EventType::ZOOM => Some(egui::Event::Zoom(e.zoom)),
        EventType::COMPOSITION_START => Some(egui::Event::CompositionStart),
        EventType::COMPOSITION_UPDATE => Some(egui::Event::CompositionUpdate(e.composition_update)),
        EventType::TOUCH => None,
    }
}

fn modifier_from_pb_to_native(m: &Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: m.alt,
        ctrl: m.ctrl,
        shift: m.shift,
        mac_cmd: m.mac_cmd,
        command: m.command,
    }
}

fn texture_id_from_native_to_pb(tid: &egui::TextureId) -> TextureId {
    let mut pbid = TextureId::default();
    match tid {
        egui::TextureId::Managed(id) => {
            pbid.tag = 1;
            pbid.managed = *id;
        }
        egui::TextureId::User(id) => {
            pbid.tag = 2;
            pbid.user = *id;
        }
    }
    pbid
}

fn rect_from_native_to_pb(rect: egui::Rect) -> Rect {
    let mut r = Rect::default();
    let min = r.min.mut_or_insert_default();
    min.x = rect.min.x;
    min.y = rect.min.y;
    let max = r.max.mut_or_insert_default();
    max.x = rect.max.x;
    max.y = rect.max.y;
    r
}

fn rect_from_pb_to_native(rect: &Rect) -> egui::Rect {
    egui::Rect {
        min: rect
            .min
            .as_ref()
            .map(pos2_from_pb_to_native)
            .unwrap_or_default(),
        max: rect
            .max
            .as_ref()
            .map(pos2_from_pb_to_native)
            .unwrap_or_default(),
    }
}

fn pos2_from_pb_to_native(pos: &Pos2) -> egui::Pos2 {
    egui::Pos2 { x: pos.x, y: pos.y }
}

#[no_mangle]
pub extern "C" fn end() -> Buffer {
    let output = CONTEXT.end_frame();
    let mut pb_output = Output::default();
    pb_output.repaint_after = output.repaint_after.as_millis() as u32;
    for (id, texture) in output.textures_delta.set {
        let id = texture_id_from_native_to_pb(&id);
        let mut pb_texture = Texture::default();
        *pb_texture.id.mut_or_insert_default() = id;
        pb_texture.texture_filter = match texture.filter {
            TextureFilter::Nearest => 1,
            TextureFilter::Linear => 2,
        };
        if let Some(pos) = texture.pos {
            pb_texture.pos_x = pos[0] as u32;
            pb_texture.pos_y = pos[1] as u32;
        }
        let image = pb_texture.image.mut_or_insert_default();
        match texture.image {
            ImageData::Color(color) => {
                image.width = color.size[0] as u32;
                image.height = color.size[1] as u32;
                image.data = bytemuck::cast_slice(color.pixels.as_slice()).to_vec();
            }
            ImageData::Font(font) => {
                image.width = font.size[0] as u32;
                image.height = font.size[1] as u32;
                image.data = font.srgba_pixels(1.0).flat_map(|a| a.to_array()).collect();
            }
        }
        pb_output.texture_set.push(pb_texture);
    }
    pb_output.texture_free = output
        .textures_delta
        .free
        .iter()
        .map(texture_id_from_native_to_pb)
        .collect();
    let clipped = CONTEXT.tessellate(output.shapes);
    for cp in clipped {
        let mut pb_cp = ClippedPrimitive::default();
        *pb_cp
            .clip_rect
            .mut_or_insert_default() =
            rect_from_native_to_pb(cp.clip_rect);
        match cp.primitive {
            Primitive::Mesh(mesh) => {
                pb_cp.tag = 1;
                let mut pb_mesh = Mesh::default();
                pb_mesh.indices = mesh.indices;
                *pb_mesh
                    .texture_id
                    .mut_or_insert_default() =
                    texture_id_from_native_to_pb(&mesh.texture_id);
                for v in mesh.vertices {
                    let mut pbv = Vertex::default();
                    *pbv.pos.mut_or_insert_default() = Pos2 {
                        x: v.pos.x,
                        y: v.pos.y,
                        ..Default::default()
                    };
                    *pbv.uv.mut_or_insert_default() = Pos2 {
                        x: v.uv.x,
                        y: v.uv.y,
                        ..Default::default()
                    };
                    *pbv.color.mut_or_insert_default() = Color32 {
                        r: v.color[0] as u32,
                        g: v.color[1] as u32,
                        b: v.color[2] as u32,
                        a: v.color[3] as u32,
                        ..Default::default()
                    };
                }
            }
            Primitive::Callback(_) => {
                unimplemented!("callback not supported")
            }
        }
        pb_output.primitives.push(pb_cp);
    }
    let mut buffer = BUFFER.lock().unwrap();
    buffer.clear();
    pb_output.write_to_vec(&mut buffer).unwrap();
    Buffer {
        data: buffer.as_mut_ptr(),
        len: buffer.len(),
    }
}

#[no_mangle]
pub extern "C" fn update() {
    native_update(&CONTEXT);
}

pub fn native_update(context: &Context) {
    CentralPanel::default().show(context, |ui| {
        ui.label("hello, world");
    });
}
