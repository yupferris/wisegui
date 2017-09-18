extern crate minifb;

mod wisegui;

use minifb::{MouseButton, MouseMode, Scale, Window, WindowOptions};

use wisegui::*;

use std::time::Duration;
use std::thread;

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
    let mut window = Window::new("wisegui test", 512, 384, WindowOptions {
        borderless: false,
        title: true,
        resize: false,//true, // Until we can query client area size on Window consistently, see https://github.com/emoon/rust_minifb/issues/42
        scale: Scale::X2,
    }).unwrap();

    let context = Context::new(Box::new(VirtualBoyPalette));

    let mut is_done = false;

    while window.is_open() && !is_done {
        let (width, height) = window.get_size();
        let mut buffer: Vec<u32> = vec![0; width * height];

        {
            let mouse_pos = {
                let p = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
                (p.0 as i32, p.1 as i32)
            };

            let mut painter = Painter::new(&context, &mut buffer, width, height, mouse_pos, window.get_mouse_down(MouseButton::Left));

            painter.clear(Color::Dark);

            painter.rect(4, 4, (width - 8) as _, (height - 8) as _, Color::Darkest, Color::Light);

            let pattern_offset = (100, 100);
            let pattern_width = 16;
            let pattern_height = 16;
            for y in 0..pattern_height {
                for x in 0..pattern_width {
                    let value = (y << 4) | x;
                    painter.text(pattern_offset.0 + x * 20, pattern_offset.1 + y * 8, if ((x + y) & 0x01) == 0 { Color::Light } else { Color::Lightest }, &format!("{:02x}", value));
                }
            }

            let mut layout = painter.stack_vertical(8, 8);

            layout.text(Color::Light, "here's something <(-.-)> :D");
            layout.text(Color::Lightest, "Holy what, it works!!");
            layout.button("here's a button that does nothing...");
            layout.text(Color::Lightest, "stack stack stack...");
            if layout.button("PUSH ME YO") {
                is_done = true;
            }
            layout.text(Color::Light, "Continue stacking..");
        }

        window.update_with_buffer(&buffer).unwrap();

        thread::sleep(Duration::from_millis(1));
    }
}
