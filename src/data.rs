use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    utils::HashMap,
};
use bitmatch::bitmatch;
// use noise::{NoiseFn, Perlin};

/// Represents the variant of any voxel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Voxel {
    Stone,
    Dirt,
    Grass,
}

impl Voxel {
    /// Size in logical units
    pub const WORLD_SIZE: f32 = 1.0;

    /// Half of logical size
    pub const HALF_SIZE: f32 = 0.5 * Self::WORLD_SIZE;

    /// Every Voxel variant in an array
    pub const ALL: &'static [Self] = &[Self::Stone, Self::Dirt, Self::Grass];

    /// How many Voxel variants there are
    pub const COUNT: usize = Self::ALL.len();

    /// Matches the Voxel type to a boolean indicator of its transparency
    pub fn is_transparent(&self) -> bool {
        false
    }
}

/// Represents an axis in 3D space
#[derive(Debug, Clone, Copy)]
enum VoxelAxis {
    X,
    Y,
    Z,
}

/// Represents the direction of an axis
#[derive(Debug, Clone, Copy)]
enum VoxelSign {
    Neg,
    Pos,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
/// TODO: This is probably an overengineered solution to save space
///
/// A table of the positions, normals, and uvs for each voxel face
/// 0 means minimum; 1 means maximum
///
/// ## Positions
/// The first 4 bit triplets represent the position for
/// each of the 4 *vertices*.
///
/// ## Normals
/// The next 3 bit pairs represent the normal for each *face*,
/// where each pair is a value between [0, 2], which is translated to
/// -1.0, 0.0, or 1.0.
///
/// ## UVs
/// The last 4 bit pairs  represent the uvs for each of the 4
/// *vertices*.
enum VoxelVertices {
    NegX = 0b_001_011_010_000__00_01_01__11_10_00_01,
    NegY = 0b_101_001_000_100__01_00_01__01_11_10_00,
    NegZ = 0b_010_110_100_000__01_01_00__10_00_01_11,
    PosX = 0b_100_110_111_101__10_01_01__11_10_00_01,
    PosY = 0b_110_010_011_111__01_10_01__10_00_01_11,
    PosZ = 0b_001_101_111_011__01_01_10__01_11_10_00,
}

impl VoxelVertices {
    /// Returns the corresponding `VoxelAxis` for the current variant
    fn get_axis(&self) -> VoxelAxis {
        match *self {
            Self::NegX | Self::PosX => VoxelAxis::X,
            Self::NegY | Self::PosY => VoxelAxis::Y,
            _ => VoxelAxis::Z,
        }
    }

    /// Returns the Corresponding `VoxelSign` for the current variant
    fn get_sign(&self) -> VoxelSign {
        match *self {
            Self::NegX | Self::NegY | Self::NegZ => VoxelSign::Neg,
            _ => VoxelSign::Pos,
        }
    }

    /// Converts the compressed format specified by the current variant
    /// into an array of vertices
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

/// Represents a Vertex in 3D space
#[derive(Debug, Clone, Copy, Default)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

/// Represents a face for a voxel; 4 vertices and 6 indices for 2
/// triangles
#[derive(Debug, Clone, Copy)]
struct VoxelFace {
    vertices: [Vertex; 4],
    indices: [u32; 6],
}

/// A HashMap corresponding voxels to chunk indices
#[derive(Debug, Clone, Component)]
pub struct Chunk(pub HashMap<ChunkIndex, Voxel>);

// pub struct ChunkGroup(Vec<Chunk>);

/// Indexing type
pub type ChunkIndex = u16;

/// Coordinate type inside the chunk
pub type ChunkCoord = u8;

/// Coordinate for where the chunk is compared to other chunks in
/// the world
pub type WorldChunkCoord = i32;

impl Chunk {
    pub const SEA_LEVEL: u32 = 32;

    /// The unpadded width of the chunk (the width to render)
    pub const WIDTH: ChunkCoord = 16;

    /// The chunk contains information from the surrounding chunks to
    /// help with meshing
    pub const WIDTH_PADDED: ChunkCoord = Self::WIDTH + 2;

    /// The padded width cubed
    pub const SIZE_PADDED: ChunkIndex = (Self::WIDTH_PADDED as ChunkIndex)
        * (Self::WIDTH_PADDED as ChunkIndex)
        * (Self::WIDTH_PADDED as ChunkIndex);

    /// The size that a chunk takes up in the world (unpadded)
    pub const WORLD_SIZE: f32 = Voxel::WORLD_SIZE * Self::WIDTH as f32;

    /// The size that a chunk takes up in the world (padded)
    pub const WORLD_SIZE_PADDED: f32 = Voxel::WORLD_SIZE * Self::WIDTH_PADDED as f32;

    /// Half of the world size (unpadded)
    pub const HALF_SIZE: f32 = 0.5 * Self::WORLD_SIZE;

    /// Half of the world size (padded)
    pub const HALF_SIZE_PADDED: f32 = 0.5 * Self::WORLD_SIZE_PADDED;

    /// Generate a chunk
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

            // let [x, y, z] = Self::deinterleave(i).map(|c| c as f32 - Self::HALF_SIZE_PADDED);
            // let x = x + coords[0] as f32 * Self::WORLD_SIZE;
            // let y = y + coords[1] as f32 * Self::WORLD_SIZE;
            // let z = z + coords[2] as f32 * Self::WORLD_SIZE;

            // let max_y = (if y < Self::SEA_LEVEL as f32 {
            //     1.0
            // } else {
            //     perlin.get([x as f64 * 0.001, z as f64 * 0.001]) - Self::HALF_SIZE_PADDED as f64
            // } * Self::WIDTH_PADDED as f64) as f32
            //     - Self::HALF_SIZE_PADDED;

            // if y < max_y {
            //     voxels.insert(i, Voxel::Stone);
            // }
            // voxels.insert(i, Voxel::Stone);
        }
        Self(voxels)
    }

    /// Pack three `ChunkCoord`s into one `ChunkIndex`
    #[bitmatch]
    fn interleave(x: ChunkCoord, y: ChunkCoord, z: ChunkCoord) -> ChunkIndex {
        bitpack!("xyz_xyz_xyz_xyz_xyz")
    }

    /// Unpack a `ChunkIndex` into three `ChunkCoord`s
    #[bitmatch]
    fn deinterleave(i: ChunkIndex) -> [ChunkCoord; 3] {
        #[bitmatch]
        let "xyz_xyz_xyz_xyz_xyz" = i;
        [x as ChunkCoord, y as ChunkCoord, z as ChunkCoord]
    }

    /// Check if a coordinate is part of the core chunk
    fn coord_is_not_padding(coord: ChunkCoord) -> bool {
        coord > 0 && coord < Self::WIDTH_PADDED - 1
    }

    /// Performs face culling on all the voxels in a chunk
    /// (non-padding) and returns a list of vertices with their
    /// corresponding indices
    fn face_culling(&self) -> Vec<VoxelFace> {
        // The vector to return
        let mut faces = Vec::new();

        // The current index
        // Used to return the correct indices for each voxel face
        let mut index = 0;

        for i in 0..Self::SIZE_PADDED {
            if let Some(voxel) = self.0.get(&i) {
                let [x, y, z] = Self::deinterleave(i);

                // Only render non-padding voxels
                if !(Self::coord_is_not_padding(x)
                    && Self::coord_is_not_padding(y)
                    && Self::coord_is_not_padding(z))
                {
                    continue;
                }

                // Array of tuples to represent which faces to render
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

                // Add all rendered faces to the final list
                for (vertices, should_render) in should_render {
                    if should_render {
                        faces.push(VoxelFace {
                            // Decompress each face
                            vertices: vertices.to_vertex_array(i, *voxel),

                            // Two triangles
                            indices: [index, index + 1, index + 2, index + 2, index + 3, index],
                        });

                        // Use next four indices next time
                        index += 4;
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

        // Retrieve positions, normals, uvs, and indices from list of faces
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
