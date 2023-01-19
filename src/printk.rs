use spin::Mutex;
use core::fmt::{self, Write};
use bootloader_api::BootInfo;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight};
use noto_sans_mono_bitmap::{get_raster, get_raster_width};
use conquer_once::spin::OnceCell;

const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const FONT_HEIGHT: RasterHeight = RasterHeight::Size32;
const FONT_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_HEIGHT);

pub static PRINTK: OnceCell<Mutex<Printk>> = OnceCell::uninit();

pub fn init(boot_info: &'static mut BootInfo) {
    let buffer_optional = &mut boot_info.framebuffer;
    let buffer = buffer_optional.as_mut().unwrap();
    
    PRINTK.try_init_once(|| Mutex::new(Printk {
        row_position: 0,
        column_position: 0,
        info: buffer.info().clone(),
        buffer: buffer.buffer_mut(),
        level: PrintLevel::Info,
    })).unwrap();
    PRINTK.try_get().unwrap().lock().clear_screen();
}

pub struct Printk {
    row_position: usize,
    column_position: usize,
    info: FrameBufferInfo,
    buffer: &'static mut [u8],
    level: PrintLevel,
}

#[derive(Debug)]
pub enum PrintLevel {
    Debug,
    Info,
    Warn,
    Error,
    None
}

impl PrintLevel {
    pub fn name(&self) -> &'static str {
        match self {
            PrintLevel::Debug => "DEBUG",
            PrintLevel::Info => "INFO",
            PrintLevel::Warn => "WARN",
            PrintLevel::Error => "ERROR",
            _ => panic!("{:?} is not a valid PrintLevel", self),
        }
    }
    const fn get_color_rgb(&self) -> [u8; 3] {
        match self {
            PrintLevel::Debug => [0x03, 0xa9, 0xf4],
            PrintLevel::Info => [0x4c, 0xaf, 0x50],
            PrintLevel::Warn => [0xff, 0xc1, 0x07],
            PrintLevel::Error => [0xf4, 0x43, 0x36],
            PrintLevel::None => [0xff, 0xff, 0xff],
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
    pub fn draw_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        let write_range = byte_offset..(byte_offset + bytes_per_pixel);
        let color = self.level.get_color_pixel(self.info.pixel_format, intensity);
        self.buffer[write_range].copy_from_slice(&color[..bytes_per_pixel]);
    }

    fn new_line(&mut self) {
        self.row_position = 0;
        self.column_position += FONT_HEIGHT as usize;
    }

    fn clear_screen(&mut self) {
        self.buffer.fill(0);
        self.row_position = 0;
        self.column_position = 0;
    }

    pub fn write_byte(&mut self, byte: char) {
        match byte {
            '\n' => self.new_line(),
            _ => {
                if self.row_position >= self.info.width - FONT_WIDTH {
                    self.new_line();
                }
                if self.column_position >= self.info.height {
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
    }

    pub fn change_level(&mut self, level: PrintLevel) {
        self.level = level;
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        PRINTK.try_get().unwrap().lock().write_fmt(args).unwrap();
    });
}

pub fn change_print_level(level: PrintLevel) {
    PRINTK.try_get().unwrap().lock().change_level(level);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::printk::change_print_level($crate::printk::PrintLevel::None);
        $crate::printk::_print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => ({
        $crate::printk::change_print_level($level);
        $crate::printk::_print(format_args!("[{}] {}\n", $level.name(), format_args!($($arg)*)));
    });
}