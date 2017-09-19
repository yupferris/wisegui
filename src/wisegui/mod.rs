extern crate image;

use self::image::{GenericImage, Pixel};

const FONT_CHAR_WIDTH: usize = 8;
const FONT_CHAR_HEIGHT: usize = 8;

pub struct Font {
    chars: Vec<Vec<u8>>,
}

impl Font {
    fn new() -> Font {
        let data = include_bytes!("font.png");
        let image = image::load_from_memory(data).unwrap();

        let mut chars = Vec::new();

        for char_y in 0..(image.height() as usize) / FONT_CHAR_HEIGHT {
            for char_x in 0..(image.width() as usize) / FONT_CHAR_WIDTH {
                let mut char_vec = Vec::new();

                for y in 0..FONT_CHAR_HEIGHT {
                    let mut acc = 0;

                    for x in 0..FONT_CHAR_WIDTH {
                        let pixel_x = char_x * FONT_CHAR_WIDTH + x;
                        let pixel_y = char_y * FONT_CHAR_HEIGHT + y;

                        let pixel = image.get_pixel(pixel_x as _, pixel_y as _).to_rgb();

                        acc <<= 1;
                        acc |= if pixel.data[0] > 0 { 1 } else { 0 };
                    }

                    char_vec.push(acc);
                }

                chars.push(char_vec);
            }
        }

        Font {
            chars: chars,
        }
    }

    pub fn measure(string: &str) -> (i32, i32) {
        ((string.len() * FONT_CHAR_WIDTH) as i32, FONT_CHAR_HEIGHT as i32)
    }
}

#[derive(Clone, Copy)]
pub enum Color {
    Darkest,
    Dark,
    Light,
    Lightest,
}

pub trait Palette {
    fn color(&self, color: Color) -> u32;
}

pub struct Context {
    font: Font,
    palette: Box<Palette>,

    mouse_pos: (i32, i32),

    is_left_mouse_down: bool,
    last_is_left_mouse_down: bool,

    was_left_mouse_pressed: bool,
    left_mouse_pressed_pos: (i32, i32),

    was_left_mouse_released: bool,
    left_mouse_released_pos: (i32, i32),
}

impl Context {
    pub fn new(palette: Box<Palette>) -> Context {
        Context {
            font: Font::new(),
            palette: palette,

            mouse_pos: (0, 0),

            is_left_mouse_down: false,
            last_is_left_mouse_down: false,

            was_left_mouse_pressed: false,
            left_mouse_pressed_pos: (0, 0),

            was_left_mouse_released: false,
            left_mouse_released_pos: (0, 0),
        }
    }

    pub fn update(&mut self, mouse_pos: (i32, i32), is_left_mouse_down: bool) {
        self.mouse_pos = mouse_pos;

        self.was_left_mouse_pressed = is_left_mouse_down && !self.last_is_left_mouse_down;
        if self.was_left_mouse_pressed {
            self.left_mouse_pressed_pos = mouse_pos;
        }

        self.was_left_mouse_released = !is_left_mouse_down && self.last_is_left_mouse_down;
        if self.was_left_mouse_released {
            self.left_mouse_released_pos = mouse_pos;
        }

        self.last_is_left_mouse_down = self.is_left_mouse_down;

        self.is_left_mouse_down = is_left_mouse_down;
    }
}

pub struct Painter<'a> {
    context: &'a Context,
    buffer: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> Painter<'a> {
    pub fn new(context: &'a Context, buffer: &'a mut [u32], width: usize, height: usize) -> Painter<'a> {
        Painter {
            context: context,
            buffer: buffer,
            width: width,
            height: height,
        }
    }

    pub fn clear(&mut self, color: Color) {
        let color = self.context.palette.color(color);

        for p in self.buffer.iter_mut() {
            *p = color;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, fill: Color, stroke: Color) {
        {
            let mut x = x;
            let mut y = y;
            let mut width = width;
            let mut height = height;

            // Clip
            if x < 0 {
                width += x;
                x = 0;
            }
            if y < 0 {
                height += y;
                y = 0;
            }
            if x + width > (self.width as i32) {
                width -= (self.width as i32) - (x + width);
            }
            if y + height > (self.height as i32) {
                height -= (self.height as i32) - (y + height);
            }

            if x >= (self.width as i32) || y >= (self.height as i32) || width <= 0 || height <= 0 || x + width < 0 || y + height < 0 {
                return;
            }

            // Fill
            let fill = self.context.palette.color(fill);

            for pixel_y in y..y + height {
                for pixel_x in x..x + width {
                    self.buffer[(pixel_y as usize) * self.width + (pixel_x as usize)] = fill;
                }
            }
        }

        self.horizontal_line(x, width, y, stroke);
        self.horizontal_line(x, width, y + height - 1, stroke);
        self.vertical_line(y, height, x, stroke);
        self.vertical_line(y, height, x + width - 1, stroke);
    }

    pub fn horizontal_line(&mut self, mut x: i32, mut width: i32, y: i32, stroke: Color) {
        // Clip
        if y < 0 || y >= (self.height as i32) {
            return;
        }
        if x < 0 {
            width += x;
            x = 0;
        }
        if x + width > (self.width as i32) {
            width -= (self.width as i32) - (x + width);
        }

        if x >= (self.width as i32) || width <= 0 || x + width < 0 {
            return;
        }

        // Stroke
        let stroke = self.context.palette.color(stroke);

        for pixel_x in x..x + width {
            self.buffer[(y as usize) * self.width + (pixel_x as usize)] = stroke;
        }
    }

    pub fn vertical_line(&mut self, mut y: i32, mut height: i32, x: i32, stroke: Color) {
        // Clip
        if x < 0 || x >= (self.width as i32) {
            return;
        }
        if y < 0 {
            height += y;
            y = 0;
        }
        if y + height > (self.height as i32) {
            height -= (self.height as i32) - (y + height);
        }

        if y >= (self.height as i32) || height <= 0 || y + height < 0 {
            return;
        }

        // Stroke
        let stroke = self.context.palette.color(stroke);

        for pixel_y in y..y + height {
            self.buffer[(pixel_y as usize) * self.width + (x as usize)] = stroke;
        }
    }

    pub fn text(&mut self, mut x: i32, y: i32, color: Color, string: &str) {
        let color = self.context.palette.color(color);

        for c in string.chars() {
            let min_ascii_code = 32;
            let max_ascii_code = min_ascii_code + (self.context.font.chars.len() as u32);

            let mut ascii_code = c as u32;
            if ascii_code < min_ascii_code || ascii_code >= max_ascii_code {
                ascii_code = 32; // Default to space
            }

            let char_bytes = &self.context.font.chars[(ascii_code - 32) as usize];

            for char_y in 0..FONT_CHAR_HEIGHT {
                let char_byte = char_bytes[char_y];
                for char_x in 0..FONT_CHAR_WIDTH {
                    if (char_byte >> (7 - char_x)) & 0x01 == 0 {
                        continue;
                    }

                    let pixel_x = x + (char_x as i32);
                    let pixel_y = y + (char_y as i32);

                    if pixel_x < 0 || pixel_y < 0 || pixel_x >= (self.width as i32) || pixel_y >= (self.height as i32) {
                        continue;
                    }

                    self.buffer[(pixel_y as usize) * self.width + (pixel_x as usize)] = color;
                }
            }

            x += FONT_CHAR_WIDTH as i32;
        }
    }

    pub fn stack_vertical(&'a mut self, x: i32, y: i32) -> VerticalStackLayout<'a> {
        VerticalStackLayout::new(self, x, y)
    }
}

pub struct VerticalStackLayout<'a> {
    painter: &'a mut Painter<'a>,
    cursor: (i32, i32),
}

impl<'a> VerticalStackLayout<'a> {
    fn new(painter: &'a mut Painter<'a>, x: i32, y: i32) -> VerticalStackLayout<'a> {
        VerticalStackLayout {
            painter: painter,
            cursor: (x, y),
        }
    }

    pub fn text(&mut self, color: Color, string: &str) {
        self.painter.text(self.cursor.0, self.cursor.1, color, string);
        let font_metrics = Font::measure(string);
        self.cursor.1 += font_metrics.1;
    }

    pub fn button(&mut self, string: &str) -> bool {
        let padding = 4;

        let font_metrics = Font::measure(string);

        let total_size = (font_metrics.0 + padding * 2, font_metrics.1 + padding * 2);

        fn is_in_bounds(pos: (i32, i32), size: (i32, i32), point: (i32, i32)) -> bool {
            point.0 >= pos.0 &&
            point.0 < pos.0 + size.0 &&
            point.1 >= pos.1 &&
            point.1 < pos.1 + size.1
        }

        let is_mouse_pos_in_bounds = is_in_bounds(self.cursor, total_size, self.painter.context.mouse_pos);
        let is_left_pressed_pos_in_bounds = is_in_bounds(self.cursor, total_size, self.painter.context.left_mouse_pressed_pos);
        let is_hovered = is_mouse_pos_in_bounds && !self.painter.context.is_left_mouse_down;
        let is_down = is_mouse_pos_in_bounds && self.painter.context.is_left_mouse_down && is_left_pressed_pos_in_bounds;
        let was_pressed =
            self.painter.context.was_left_mouse_released &&
            is_left_pressed_pos_in_bounds &&
            is_in_bounds(self.cursor, total_size, self.painter.context.left_mouse_released_pos);

        let bg_color = if is_down { Color::Light } else { if is_hovered { Color::Dark } else { Color::Darkest } };
        let text_color = if is_down { Color::Darkest } else { Color::Lightest };

        self.painter.rect(self.cursor.0, self.cursor.1, total_size.0, total_size.1, bg_color, Color::Light);
        self.painter.text(self.cursor.0 + padding, self.cursor.1 + padding, text_color, string);

        self.cursor.1 += total_size.1;

        was_pressed
    }
}
