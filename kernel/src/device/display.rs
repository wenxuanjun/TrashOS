use core::slice::from_raw_parts_mut;
use limine::request::FramebufferRequest;
use os_terminal::{DrawTarget, Rgb888};

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    U8,
    Unknown,
}

pub struct Display {
    buffer: &'static mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    bytes_per_pixel: usize,
    pixel_format: PixelFormat,
}

impl Display {
    pub fn new() -> Self {
        let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let frame_buffer = response.framebuffers().next().take().unwrap();

        let width = frame_buffer.width() as _;
        let height = frame_buffer.height() as _;

        let pixel_format = match (
            frame_buffer.red_mask_shift(),
            frame_buffer.green_mask_shift(),
            frame_buffer.blue_mask_shift(),
        ) {
            (0x00, 0x08, 0x10) => PixelFormat::Rgb,
            (0x10, 0x08, 0x00) => PixelFormat::Bgr,
            (0x00, 0x00, 0x00) => PixelFormat::U8,
            _ => PixelFormat::Unknown,
        };

        let pitch = frame_buffer.pitch() as usize;
        let bpp = frame_buffer.bpp() as usize;
        let stride = (pitch / 4) as _;
        let bytes_per_pixel = (bpp / 8) as _;

        let buffer_size = stride * height * bytes_per_pixel;
        let buffer = unsafe { from_raw_parts_mut(frame_buffer.addr(), buffer_size) };

        Self {
            buffer,
            width,
            height,
            stride,
            bytes_per_pixel,
            pixel_format,
        }
    }
}

impl DrawTarget for Display {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    #[inline(always)]
    fn draw_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        let byte_offset = (y * self.stride + x) * self.bytes_per_pixel;
        let write_range = byte_offset..(byte_offset + self.bytes_per_pixel);

        let color = match self.pixel_format {
            PixelFormat::Rgb => [color.0, color.1, color.2, 0],
            PixelFormat::Bgr => [color.2, color.1, color.0, 0],
            PixelFormat::U8 => unimplemented!(),
            PixelFormat::Unknown => return,
        };

        self.buffer[write_range].copy_from_slice(&color[..self.bytes_per_pixel]);
    }
}
