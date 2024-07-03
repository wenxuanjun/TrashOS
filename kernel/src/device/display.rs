use core::slice::from_raw_parts_mut;
use limine::request::FramebufferRequest;

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
    pub buffer: &'static mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
    pub pixel_format: PixelFormat,
}

impl Display {
    pub fn get() -> Self {
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
