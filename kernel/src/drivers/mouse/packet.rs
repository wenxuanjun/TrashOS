use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MouseFlags: u8 {
        const LEFT_BUTTON = 1 << 0;
        const RIGHT_BUTTON = 1 << 1;
        const MIDDLE_BUTTON = 1 << 2;
        const ALWAYS_ONE = 1 << 3;
        const X_SIGN = 1 << 4;
        const Y_SIGN = 1 << 5;
        const X_OVERFLOW = 1 << 6;
        const Y_OVERFLOW = 1 << 7;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseAdditionalFlags {
    FirstButton,
    SecondButton,
    ScrollUp,
    ScrollDown,
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct MousePacket {
    pub flags: MouseFlags,
    pub additional_flags: MouseAdditionalFlags,
    pub move_x: i16,
    pub move_y: i16,
}

impl MousePacket {
    pub const fn default() -> Self {
        MousePacket {
            flags: MouseFlags::empty(),
            additional_flags: MouseAdditionalFlags::None,
            move_x: 0,
            move_y: 0,
        }
    }
}
