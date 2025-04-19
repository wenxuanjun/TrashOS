#[macro_use]
mod r#macro;

pub fn read(buffer: *mut u8, length: usize) -> isize {
    syscall!(0isize, buffer as usize, length)
}

pub fn write(buffer: *const u8, length: usize) -> isize {
    syscall!(1isize, buffer as usize, length)
}

pub fn mmap(address: usize, length: usize) -> isize {
    syscall!(2isize, address, length)
}

pub fn r#yield() -> isize {
    syscall!(3isize)
}

pub fn sleep(duration: usize) -> isize {
    syscall!(4isize, duration)
}

pub fn exit() -> ! {
    syscall!(@noret 5isize)
}
