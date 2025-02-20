use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    utils::HashMap,
};
use bitmatch::bitmatch;
// use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Voxel {
    Stone,
    Dirt,
    Grass,
}

impl Voxel {
    pub const WORLD_SIZE: f32 = 1.0;
    pub const HALF_SIZE: f32 = 0.5 * Self::WORLD_SIZE;
    pub const ALL: &'static [Self] = &[Self::Stone, Self::Dirt, Self::Grass];
    pub const COUNT: usize = Self::ALL.len();

    pub fn is_transparent(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
enum VoxelAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
enum VoxelSign {
    Neg,
    Pos,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum VoxelVertices {
    NegX = 0b_001_011_010_000__00_01_01__11_10_00_01,
    NegY = 0b_101_001_000_100__01_00_01__01_11_10_00,
    NegZ = 0b_010_110_100_000__01_01_00__10_00_01_11,
    PosX = 0b_100_110_111_101__10_01_01__11_10_00_01,
    PosY = 0b_110_010_011_111__01_10_01__10_00_01_11,
    PosZ = 0b_001_101_111_011__01_01_10__01_11_10_00,
}

impl VoxelVertices {
    fn get_axis(&self) -> VoxelAxis {
        match *self {
            Self::NegX | Self::PosX => VoxelAxis::X,
            Self::NegY | Self::PosY => VoxelAxis::Y,
            _ => VoxelAxis::Z,
        }
    }

    fn get_sign(&self) -> VoxelSign {
        match *self {
            Self::NegX | Self::NegY | Self::NegZ => VoxelSign::Neg,
            _ => VoxelSign::Pos,
        }
    }

    #[bitmatch]
    fn to_vertex_array(&self, index: ChunkIndex, voxel: Voxel) -> [Vertex; 4] {
        let vhs = Voxel::HALF_SIZE;
        let chs = Chunk::HALF_SIZE;
        let [x, y, z] = Chunk::deinterleave(index).map(|c| c as f32 - vhs - chs);

        #[bitmatch]
        let "abc_def_ghi_jkl__mm_nn_oo__pq_rs_tu_vw" = *self as u32;

        let unit_u = (Voxel::COUNT as f32).recip();
        let unit_v = 6.0f32.recip();
        let uv_u = (voxel as u16) as f32 * unit_u;
        let uv_v = (3.0 * self.get_sign() as u8 as f32 + self.get_axis() as u8 as f32) * unit_v;

        [
            Vertex {
                position: [a as f32 + x, b as f32 + y, c as f32 + z],
                normal: [m as f32 - 1.0, n as f32 - 1.0, o as f32 - 1.0],
                uv: [uv_u + p as f32 * unit_u, uv_v + q as f32 * unit_v],
            },
            Vertex {
                position: [d as f32 + x, e as f32 + y, f as f32 + z],
                normal: [m as f32 - 1.0, n as f32 - 1.0, o as f32 - 1.0],
                uv: [uv_u + r as f32 * unit_u, uv_v + s as f32 * unit_v],
            },
            Vertex {
                position: [g as f32 + x, h as f32 + y, i as f32 + z],
                normal: [m as f32 - 1.0, n as f32 - 1.0, o as f32 - 1.0],
                uv: [uv_u + t as f32 * unit_u, uv_v + u as f32 * unit_v],
            },
            Vertex {
                position: [j as f32 + x, k as f32 + y, l as f32 + z],
                normal: [m as f32 - 1.0, n as f32 - 1.0, o as f32 - 1.0],
                uv: [uv_u + v as f32 * unit_u, uv_v + w as f32 * unit_v],
            },
        ]
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
struct VoxelFace {
    vertices: [Vertex; 4],
    indices: [u32; 6],
}

#[derive(Debug, Clone, Component)]
pub struct Chunk(pub HashMap<ChunkIndex, Voxel>);

// pub struct ChunkGroup(Vec<Chunk>);

pub type ChunkIndex = u16;
pub type ChunkCoord = u8;
pub type WorldChunkCoord = i32;

impl Chunk {
    pub const SEA_LEVEL: u32 = 32;
    pub const WIDTH: ChunkCoord = 16;
    pub const WIDTH_PADDED: ChunkCoord = Self::WIDTH + 2;
    pub const SIZE_PADDED: ChunkIndex = (Self::WIDTH_PADDED as ChunkIndex)
        * (Self::WIDTH_PADDED as ChunkIndex)
        * (Self::WIDTH_PADDED as ChunkIndex);

    pub const WORLD_SIZE: f32 = Voxel::WORLD_SIZE * Self::WIDTH as f32;
    pub const HALF_SIZE: f32 = 0.5 * Self::WORLD_SIZE;

    // pub fn new() -> Self {
    pub fn new(_seed: u32, _coords: [WorldChunkCoord; 3]) -> Self {
        let mut voxels = HashMap::new();
        // let perlin = Perlin::new(seed);
        for i in 0..Self::SIZE_PADDED {
            // Circle
            let [x, y, z] = Self::deinterleave(i).map(|c| c as f32 - Self::HALF_SIZE);
            if (x * x + y * y + z * z).sqrt() < Self::HALF_SIZE {
                if y < Self::HALF_SIZE * 0.5 {
                    voxels.insert(i, Voxel::Stone);
                } else {
                    voxels.insert(i, Voxel::Dirt);
                }
            }
            // let [x, y, z] = Self::deinterleave(i).map(|c| c as f32 - Self::HALF_SIZE);
            // let x = x + coords[0] as f32 * Self::WORLD_SIZE;
            // let z = z + coords[2] as f32 * Self::WORLD_SIZE;

            // let max_y = (if (coords[1] * Self::WIDTH as i32) < Self::SEA_LEVEL as i32 {
            //     1.0
            // } else {
            //     perlin.get([x as f64 * 0.001, z as f64 * 0.001]) - Self::HALF_SIZE as f64
            // } * Self::WIDTH_PADDED as f64) as f32
            //     - Self::HALF_SIZE;

            // if y < max_y {
            //     voxels.insert(i, Voxel::Stone);
            // }
        }
        Self(voxels)
    }

    #[bitmatch]
    fn interleave(x: ChunkCoord, y: ChunkCoord, z: ChunkCoord) -> ChunkIndex {
        bitpack!("xyz_xyz_xyz_xyz_xyz")
    }

    #[bitmatch]
    fn deinterleave(i: ChunkIndex) -> [ChunkCoord; 3] {
        #[bitmatch]
        let "xyz_xyz_xyz_xyz_xyz" = i;
        [x as ChunkCoord, y as ChunkCoord, z as ChunkCoord]
    }

    fn coord_is_not_padding(coord: ChunkCoord) -> bool {
        coord > 0 && (coord as ChunkIndex) < Self::SIZE_PADDED - 1
    }

    fn face_culling(&self) -> Vec<VoxelFace> {
        let mut faces = Vec::new();
        let mut index = 0;
        for i in 0..Self::SIZE_PADDED {
            if let Some(voxel) = self.0.get(&i) {
                let [x, y, z] = Self::deinterleave(i);
                if Self::coord_is_not_padding(x)
                    && Self::coord_is_not_padding(y)
                    && Self::coord_is_not_padding(z)
                {
                    let should_render = [
                        (
                            VoxelVertices::NegX,
                            self.0
                                .get(&Self::interleave(x - 1, y, z))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                        (
                            VoxelVertices::NegY,
                            self.0
                                .get(&Self::interleave(x, y - 1, z))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                        (
                            VoxelVertices::NegZ,
                            self.0
                                .get(&Self::interleave(x, y, z - 1))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                        (
                            VoxelVertices::PosX,
                            self.0
                                .get(&Self::interleave(x + 1, y, z))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                        (
                            VoxelVertices::PosY,
                            self.0
                                .get(&Self::interleave(x, y + 1, z))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                        (
                            VoxelVertices::PosZ,
                            self.0
                                .get(&Self::interleave(x, y, z + 1))
                                .is_none_or(|v| v.is_transparent()),
                        ),
                    ];
                    for (vertices, should_render) in should_render {
                        if should_render {
                            faces.push(VoxelFace {
                                vertices: vertices.to_vertex_array(i, *voxel),
                                indices: [index, index + 1, index + 2, index + 2, index + 3, index],
                            });
                            index += 4;
                        }
                    }
                }
            }
        }
        faces
    }
}

impl From<Chunk> for Mesh {
    fn from(chunk: Chunk) -> Self {
        let faces = chunk.face_culling();
        let positions: Vec<_> = faces
            .iter()
            .map(|f| f.vertices.iter().map(|v| v.position).collect::<Vec<_>>())
            .flatten()
            .collect();
        let normals: Vec<_> = faces
            .iter()
            .map(|f| f.vertices.iter().map(|v| v.normal).collect::<Vec<_>>())
            .flatten()
            .collect();
        let uvs: Vec<_> = faces
            .iter()
            .map(|f| f.vertices.iter().map(|v| v.uv).collect::<Vec<_>>())
            .flatten()
            .collect();
        let indices: Vec<_> = faces.iter().map(|f| f.indices).flatten().collect();

        Self::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Self::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Self::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Self::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }
}
