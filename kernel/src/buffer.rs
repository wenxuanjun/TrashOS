use spin::Mutex;
use alloc::boxed::Box;
use bootloader_api::info::FrameBufferInfo;
use conquer_once::spin::OnceCell;

static BUFFER: OnceCell<Mutex<Box<[u8]>>> = OnceCell::uninit();

struct Buffer {
    info: FrameBufferInfo,
    back_buffer: [u8],
    buffer: &'static mut [u8],
}

pub fn init(boot_info: &'static BootInfo) {
    let boot_info = boot_info as *const BootInfo as *mut BootInfo;
    let boot_info_mut = unsafe {&mut *(boot_info)};
    let frame_buffer = boot_info_mut.framebuffer.as_mut().unwrap();

    let buffer = Buffer {
        info: frame_buffer.info().clone(),
        buffer: frame_buffer.buffer_mut(),
        back_buffer: Box::new([0; frame_buffer.info().byte_len]),
    };

    BUFFER.try_init_once(|| Mutex::new(buffer)).unwrap();
}

pub fn set_pixel(x: usize, y: usize, data: &[u8]) {
    let buffer = BUFFER.try_get().unwrap().lock();
    let index = (y * buffer.width + x) * buffer.info.bytes_per_pixel;
    buffer.back_buffer[index..index + buffer.info.bytes_per_pixel].copy_from_slice(data);
}

pub fn clear_screen() {
    let buffer = BUFFER.try_get().unwrap().lock();
    buffer.back_fuffer.fill(0);
}

pub fn set_range(x: usize, y: usize, width: usize, height: usize, data: &[u8]) {
    let buffer = BUFFER.try_get().unwrap().lock();
    let start = (y * buffer.info.width + x) * buffer.info.bytes_per_pixel;
    let end = start + (width * buffer.info.bytes_per_pixel);
    back_buffer[start..end].copy_from_slice(data);
}

pub fn swap_buffers() {
    let buffer = BUFFER.try_get().unwrap().lock();
    buffer.buffer.copy_from_slice(&buffer.back_buffer);
}