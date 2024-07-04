use core::fmt::{self, Write};
use noto_sans_mono_bitmap::{get_raster, get_raster_width};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight};
use spin::{Lazy, Mutex};

use crate::device::display::{Display, PixelFormat};

const FONT_WEIGHT: FontWeight = FontWeight::Bold;
const FONT_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_HEIGHT);
const FONT_HEIGHT: RasterHeight = RasterHeight::Size16;
pub const DEFAULT_COLOR: Color = Color::White;

static PRINTK: Lazy<Mutex<Printk>> = Lazy::new(|| {
    Mutex::new(Printk {
        row_position: 0,
        column_position: 0,
        color: DEFAULT_COLOR,
        display: Display::get(),
    })
});

pub struct Printk {
    row_position: usize,
    column_position: usize,
    color: Color,
    display: Display,
}

#[derive(Debug)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    White,
}

impl Color {
    const fn get_color_rgb(&self) -> [u8; 3] {
        match self {
            Color::Red => [0xf4, 0x43, 0x36],
            Color::Yellow => [0xff, 0xc1, 0x07],
            Color::Green => [0x4c, 0xaf, 0x50],
            Color::Blue => [0x03, 0xa9, 0xf4],
            Color::White => [0xff, 0xff, 0xff],
        }
    }

    fn get_color_pixel(&self, pixel_format: PixelFormat, intensity: u8) -> [u8; 4] {
        let [r, g, b] = self
            .get_color_rgb()
            .map(|x| (x as u32 * intensity as u32 / 0xff) as u8);
        match pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            PixelFormat::U8 => [intensity >> 4, 0, 0, 0],
            _ => panic!("Unknown pixel format: {:?}", pixel_format),
        }
    }
}

impl Printk {
    fn draw_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let Display {
            pixel_format,
            bytes_per_pixel,
            stride,
            ..
        } = self.display;

        let byte_offset = (y * stride + x) * bytes_per_pixel;
        let write_range = byte_offset..(byte_offset + bytes_per_pixel);

        let color = self.color.get_color_pixel(pixel_format, intensity);
        self.display.buffer[write_range].copy_from_slice(&color[..bytes_per_pixel]);
    }

    #[inline]
    fn clear_screen(&mut self) {
        self.display.buffer.fill(0);
        self.row_position = 0;
        self.column_position = 0;
    }

    #[inline]
    fn new_line(&mut self) {
        self.row_position = 0;
        self.column_position += FONT_HEIGHT as usize;
    }

    fn back_space(&mut self) {
        if self.row_position > 0 {
            self.row_position -= FONT_WIDTH;
        }
        for y in 0..FONT_HEIGHT as usize {
            for x in 0..FONT_WIDTH {
                self.draw_pixel(self.row_position + x, self.column_position + y, 0);
            }
        }
    }

    fn write_byte(&mut self, byte: char) {
        if self.row_position >= self.display.width - FONT_WIDTH {
            self.new_line();
        }
        if self.column_position >= self.display.height {
            self.clear_screen();
        }
        let rendered = get_raster(byte, FONT_WEIGHT, FONT_HEIGHT).unwrap();
        for (y, lines) in rendered.raster().iter().enumerate() {
            for (x, column) in lines.iter().enumerate() {
                self.draw_pixel(self.row_position + x, self.column_position + y, *column);
            }
        }
        self.row_position += rendered.width();
    }
}

impl fmt::Write for Printk {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.chars() {
            match byte {
                '\n' => self.new_line(),
                '\x08' => self.back_space(),
                _ => self.write_byte(byte),
            }
        }
        Ok(())
    }
}

#[inline]
pub fn _print(color: Color, args: fmt::Arguments) {
    let mut printk = PRINTK.lock();
    printk.color = color;
    printk.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::console::printk::_print(
            $crate::console::printk::DEFAULT_COLOR,
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
