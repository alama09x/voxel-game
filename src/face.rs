#[derive(Clone, Copy, Debug)]
pub enum Face {
    Left,
    Right,
    Down,
    Up,
    Front,
    Back,
}

impl Face {
    pub const ALL: [Self; 6] = [
        Self::Left,
        Self::Right,
        Self::Down,
        Self::Up,
        Self::Front,
        Self::Back,
    ];
}
