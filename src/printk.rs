use spin::Mutex;
use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;

use {
    core::{
        fmt::{self, Write},
        ptr,
    },
    noto_sans_mono_bitmap::{
        get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
    },
};

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

        // Raw pointer to buffer start ― that's why this is unsafe
        let _ = unsafe { ptr::read_volatile(&self.buffer[boff]) };
    }

    /// Renders characters from the `noto-sans-mono-bitmap` crate
    pub fn render(&mut self, rendered: RasterizedChar) {
        // Loop through lines
        for (y, lines) in rendered.raster().iter().enumerate() {
            // Loop through characters on each line
            for (x, column) in lines.iter().enumerate() {
                // Use above draw_grayscale method to render each character in the bitmap
                self.draw_grayscale(self.row_position + x, self.column_position + y, *column)
            }
        }

        // Increment by width of each character
        self.row_position += rendered.width();
    }

    /// Moves down by `distance` number of pixels
    pub fn move_down(&mut self, distance: usize) {
        self.column_position += distance;
    }

    /// Moves down one line
    pub fn new_line(&mut self) {
        self.row_position = 0;
        self.column_position += 24;
    }

    /// Clears the screen
    pub fn clear(&mut self) {
        self.row_position = 0;
        self.row_position = 0;
        self.buffer.fill(0);
    }

    /// Prints an individual character on the screen
    pub fn write_byte(&mut self, byte: char) {
        match byte {
            '\n' => self.new_line(),
            _ => {
                if self.row_position >= self.info.width {
                    self.new_line();
                }

                let width = get_raster_width(FontWeight::Regular, RasterHeight::Size24);
                
                if self.column_position >= (self.info.height - width) {
                    self.clear();
                }

                let mapped = get_raster(byte, FontWeight::Regular, RasterHeight::Size24).unwrap();
                self.render(mapped);
            }
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
