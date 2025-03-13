use bitflags::bitflags;

pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
}

bitflags! {
    pub struct ButtonState: u64 {
        const Left = 1;
        const Right = 1 << 1;
    }
}
