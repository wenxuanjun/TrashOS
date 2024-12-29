#[macro_use]
mod r#macro;

pub fn read(buffer: *mut u8, length: usize) -> isize {
    syscall!(0, buffer as usize, length)
}

pub fn write(buffer: *const u8, length: usize) -> isize {
    syscall!(1, buffer as usize, length)
}

pub fn mmap(address: usize, length: usize) -> isize {
    syscall!(2, address, length)
}

pub fn r#yield() -> isize {
    syscall!(3)
}

pub fn sleep(duration: usize) -> isize {
    syscall!(4, duration)
}

pub fn exit() -> ! {
    syscall!(@noret 5)
}
