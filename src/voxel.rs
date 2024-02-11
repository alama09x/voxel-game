pub const VOXEL_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Voxel {
    pub ty: VoxelType,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum VoxelType {
    #[default]
    Stone,
    Dirt,
    Grass,
}
