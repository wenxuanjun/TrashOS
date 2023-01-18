use spin::Mutex;
use core::fmt::{self, Write};
use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{get_raster, get_raster_width};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar};
use conquer_once::spin::OnceCell;

pub static PRINTK: OnceCell<Mutex<Printk>> = OnceCell::uninit();

pub fn init(boot_info: &'static mut BootInfo) {
    let buffer_optional = &mut boot_info.framebuffer;
    let buffer = buffer_optional.as_mut().unwrap();
    
    PRINTK.try_init_once(|| Mutex::new(Printk {
        row_position: 0,
        column_position: 0,
        info: buffer.info().clone(),
        buffer: buffer.buffer_mut(),
    })).unwrap();
    PRINTK.try_get().unwrap().lock().clear();
}

pub struct Printk {
    row_position: usize,
    column_position: usize,
    info: FrameBufferInfo,
    buffer: &'static mut [u8],
}

impl Printk {
    /// Draws black-and-white pixels on the screen
    pub fn draw_grayscale(&mut self, x: usize, y: usize, intensity: u8) {
        // Pixel offset
        let poff = y * self.info.stride + x;

        let u8_intensity = {
            if intensity > 200 {
                0xf
            } else {
                0
            }
        };

        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity, 0],

            PixelFormat::Bgr => [intensity, intensity, intensity, 0],

            PixelFormat::U8 => [u8_intensity, 0, 0, 0],

            //TODO: use embedded-graphics to solve this problem
            _ => panic!("Unknown pixel format"),
        };

        // Number of bytes in a pixel (4 on my machine)
        let bpp = self.info.bytes_per_pixel;

        // Byte offset: multiply bytes-per-pixel by pixel offset to obtain
        let boff = poff * bpp;

        // Copy bytes
        self.buffer[boff..(boff + bpp)].copy_from_slice(&color[..bpp]);

    }

    pub fn render(&mut self, rendered: RasterizedChar) {
        for (y, lines) in rendered.raster().iter().enumerate() {
            for (x, column) in lines.iter().enumerate() {
                self.draw_grayscale(self.row_position + x, self.column_position + y, *column)
            }
        }
        self.row_position += rendered.width();
    }

    fn new_line(&mut self) {
        self.row_position = 0;
        self.column_position += 20;
    }

    fn clear(&mut self) {
        self.row_position = 0;
        self.row_position = 0;
        self.buffer.fill(0);
    }

    fn put_char(&mut self, byte: char) {
        if self.row_position >= self.info.width {
            self.new_line();
        }

        let width = get_raster_width(FontWeight::Regular, RasterHeight::Size20);
        
        if self.column_position >= (self.info.height - width) {
            self.clear();
        }

        let mapped = get_raster(byte, FontWeight::Regular, RasterHeight::Size20).unwrap();
        self.render(mapped);
    }

    fn back_space(&mut self) {
        self.row_position -= 10;
        self.draw_grayscale(self.row_position, self.column_position, 0);
    }

    pub fn write_byte(&mut self, byte: char) {
        match byte {
            '\n' => self.new_line(),
            '\x08' => self.back_space(),
            _ => self.put_char(byte),
        }
    }
}


impl fmt::Write for Printk {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.chars() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg: tt)*) => ($crate::printk::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg: tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        PRINTK.try_get().unwrap().lock().write_fmt(args).unwrap();
    });
}
