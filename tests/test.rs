use std::ptr::null;

use egui_unity::{begin, Buffer, end, update};

#[test]
fn test() {
    let buffer = Buffer {
        data: null(),
        len: 0,
    };
    begin(buffer);
    update();
    let buffer = end();
    println!("buffer size:{}", buffer.len);
}