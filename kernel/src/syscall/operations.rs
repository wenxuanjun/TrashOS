use core::{slice, str};

pub fn write(buffer: *const u8, length: usize) {
    if length == 0 {
        return;
    }

    if let Ok(string) = unsafe {
        let slice = slice::from_raw_parts(buffer, length);
        str::from_utf8(slice)
    } {
        crate::print!("{}", string);
    };
}
