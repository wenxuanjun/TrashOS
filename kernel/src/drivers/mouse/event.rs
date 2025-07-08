#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug)]
pub enum MouseEvent {
    Moved(i16, i16),
    Scroll(isize),
    Pressed(MouseButton),
    Released(MouseButton),
}
