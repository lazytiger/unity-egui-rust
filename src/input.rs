use std::ptr::slice_from_raw_parts;

use egui::{Key, RawInput};
use egui::Event::PointerButton;
use protobuf::Message;

use crate::Buffer;
use crate::proto::common::{Pos2, Rect};
use crate::proto::input::{ButtonType, Event, EventType, Input, KeyType, Modifiers};

fn key_type_from_pb_to_native(t: KeyType) -> Option<Key> {
    match t {
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
    }
}

fn button_type_from_pb_to_native(bt: ButtonType) -> Option<egui::PointerButton> {
    match bt {
        ButtonType::BT_NONE => None,
        ButtonType::PRIMARY => Some(egui::PointerButton::Primary),
        ButtonType::SECONDARY => Some(egui::PointerButton::Secondary),
        ButtonType::MIDDLE => Some(egui::PointerButton::Middle),
        ButtonType::EXTRA1 => Some(egui::PointerButton::Extra1),
        ButtonType::EXTRA2 => Some(egui::PointerButton::Extra2),
    }
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
                    e.key.enum_value().ok().map(key_type_from_pb_to_native).unwrap_or_default()
                }).unwrap_or_default()
                .map(|kt| egui::Event::Key {
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
                    .map(button_type_from_pb_to_native)
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

pub fn parse_input(buffer: Buffer) -> RawInput {
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
            println!("event:{:?} added", event);
            input.events.push(event);
        } else {
            println!("event parse failed");
        }
    }
    input
}