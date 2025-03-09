#[derive(Clone, Copy, Debug)]
pub enum Face {
    Left,
    Right,
    Bottom,
    Top,
    Back,
    Front,
}

impl Face {
    pub const fn sign(&self) -> bool {
        match self {
            Self::Left | Self::Bottom | Self::Back => false,
            _ => true,
        }
    }

    pub const fn normal(&self) -> [f32; 3] {
        match self {
            Self::Left => [-1.0, 0.0, 0.0],
            Self::Right => [1.0, 0.0, 0.0],
            Self::Bottom => [0.0, -1.0, 0.0],
            Self::Top => [0.0, 1.0, 0.0],
            Self::Back => [0.0, 0.0, -1.0],
            Self::Front => [0.0, 0.0, 1.0],
        }
    }

    pub const fn positions(&self, [x0, y0, z0]: [f32; 3], [x1, y1, z1]: [f32; 3]) -> [[f32; 3]; 4] {
        match self {
            Self::Left => [[x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [x0, y0, z0]],
            Self::Right => [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]],
            Self::Bottom => [[x1, y0, z1], [x0, y0, z1], [x0, y0, z0], [x1, y0, z0]],
            Self::Top => [[x1, y1, z0], [x0, y1, z0], [x0, y1, z1], [x1, y1, z1]],
            Self::Back => [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]],
            Self::Front => [[x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [x0, y0, z1]],
        }
    }

    pub const ALL: [Self; 6] = [
        Self::Left,
        Self::Right,
        Self::Bottom,
        Self::Top,
        Self::Back,
        Self::Front,
    ];
}
