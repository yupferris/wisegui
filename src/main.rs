extern crate minifb;

mod wisegui;

use minifb::{MouseButton, MouseMode, Scale, Window, WindowOptions};

use wisegui::*;

use std::time::Duration;
use std::thread;

/*struct DefaultPalette;

impl Palette for DefaultPalette {
    fn color(&self, color: Color) -> u32 {
        match color {
            Color::Darkest => 0x000000,
            Color::Dark => 0x555555,
            Color::Light => 0xaaaaaa,
            Color::Lightest => 0xffffff,
        }
    }
}*/

struct VirtualBoyPalette;

impl Palette for VirtualBoyPalette {
    fn color(&self, color: Color) -> u32 {
        match color {
            Color::Darkest => 0x000000,
            Color::Dark => 0x550000,
            Color::Light => 0xaa0000,
            Color::Lightest => 0xff0000,
        }
    }
}

fn main() {
    let mut window = Window::new("wisegui test", 1280, 720, WindowOptions {
        borderless: false,
        title: true,
        resize: true,
        scale: Scale::X1,
    }).unwrap();

    let mut context = Context::new(Box::new(VirtualBoyPalette));

    let mut is_done = false;

    while window.is_open() && !is_done {
        let mouse_pos = {
            let p = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
            (p.0 as i32, p.1 as i32)
        };
        let is_left_mouse_down = window.get_mouse_down(MouseButton::Left);
        context.update(mouse_pos, is_left_mouse_down);

        let (width, height) = window.get_size();
        let mut buffer: Vec<u32> = vec![0; width * height];

        {
            let mut painter = Painter::new(&context, &mut buffer, width, height);

            painter.clear(Color::Dark);

            painter.rect(4, 4, (width - 8) as _, (height - 8) as _, Color::Darkest, Color::Light);

            let pattern_offset = (420, 100);
            let pattern_width = 16;
            let pattern_height = 16;
            for y in 0..pattern_height {
                for x in 0..pattern_width {
                    let value = (y << 4) | x;
                    painter.text(pattern_offset.0 + x * 20, pattern_offset.1 + y * 8, if ((x + y) & 0x01) == 0 { Color::Light } else { Color::Lightest }, &format!("{:02x}", value));
                }
            }

            let mut cursor = (8, 8);

            painter.text(cursor.0, cursor.1, Color::Light, "here's something <(-.-)> :D");
            cursor.1 += FONT_CHAR_HEIGHT as i32;
            painter.text(cursor.0, cursor.1, Color::Lightest, "Holy what, it works!!");
            cursor.1 += FONT_CHAR_HEIGHT as i32;

            for _ in 0..40 {
                painter.button(cursor.0, cursor.1, "here are some buttons that don't do anything...");
                cursor.1 += (FONT_CHAR_HEIGHT as i32) * 2;
            }

            painter.text(cursor.0, cursor.1, Color::Lightest, "stack stack stack...");
            cursor.1 += FONT_CHAR_HEIGHT as i32;

            if painter.button(cursor.0, cursor.1, "PUSH ME YO") {
                is_done = true;
            }
            cursor.1 += (FONT_CHAR_HEIGHT as i32) * 2;

            painter.text(cursor.0, cursor.1, Color::Light, "Continue stacking..");
            cursor.1 += FONT_CHAR_HEIGHT as i32;
        }

        window.update_with_buffer(&buffer).unwrap();

        thread::sleep(Duration::from_millis(1));
    }
}
