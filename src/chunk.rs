use std::{array, iter};

use crate::{block::Block, face::Face};

use super::voxel::Voxel;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use noise::{NoiseFn, Perlin, Simplex, Worley};
use serde::{Deserialize, Serialize};

pub const CHUNK_WIDTH: u8 = 16;
pub const CHUNK_SIZE: usize = (CHUNK_WIDTH as usize).pow(3);

#[derive(Component, Clone)]
#[require(ChunkPos, ChunkNeighbors)]
pub struct Chunk<V: Voxel>(pub Box<[V; CHUNK_SIZE]>);

impl<V: Voxel> Default for Chunk<V> {
    fn default() -> Self {
        Self(Box::new(array::from_fn(|_| V::default())))
    }
}

#[derive(Component, Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub [i32; 3]);

impl ChunkPos {
    pub fn offsets(&self) -> [Self; 6] {
        let Self([cx, cy, cz]) = *self;
        [
            Self([cx - 1, cy, cz]),
            Self([cx + 1, cy, cz]),
            Self([cx, cy - 1, cz]),
            Self([cx, cy + 1, cz]),
            Self([cx, cy, cz - 1]),
            Self([cx, cy, cz + 1]),
        ]
    }
}

#[derive(Component, Default, Clone)]
pub struct ChunkNeighbors(pub [Option<Entity>; 6]);

#[derive(Component)]
pub struct ChunkNeighborsUpdateRequest;

#[derive(Component)]
pub struct ChunkMeshUpdateRequest;

#[derive(Component)]
pub struct ChunkDirty;

impl Chunk<Block> {
    pub fn generate(pos: ChunkPos, seed: u32) -> Self {
        let simplex = Simplex::new(seed);
        let worley = Worley::new(seed + 1);
        let perlin = Perlin::new(seed + 2);

        let mut data = Vec::with_capacity(CHUNK_SIZE);
        let [cx, cy, cz] = pos.0;

        // World coordinates
        let cw = CHUNK_WIDTH as f64;
        let base_x = cx as f64 * cw;
        let base_y = cy as f64 * cw;
        let base_z = cz as f64 * cw;

        let scale = 20.0;
        let octaves = 6;
        let persistence = 0.5;
        let warp_scale = 50.0;
        let warp_strength = 2.0;

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let wx = base_x + x as f64;
                    let wy = base_y + y as f64;
                    let wz = base_z + z as f64;

                    let wx_warped = wx
                        + warp_strength
                            * simplex.get([wx / warp_scale, wy / warp_scale, wz / warp_scale]);
                    let wy_warped = wy
                        + warp_strength
                            * simplex.get([
                                wx / warp_scale + 100.0,
                                wy / warp_scale + 100.0,
                                wz / warp_scale + 100.0,
                            ]);
                    let wz_warped = wz
                        + warp_strength
                            * simplex.get([
                                wx / warp_scale + 200.0,
                                wy / warp_scale + 200.0,
                                wz / warp_scale + 200.0,
                            ]);

                    let mut density = 0.0;
                    let mut frequency = 1.0;
                    let mut amplitude = 1.0;

                    for _ in 0..octaves {
                        density += amplitude
                            * simplex.get([
                                wx_warped * frequency / scale,
                                wy_warped * frequency / scale,
                                wz_warped * frequency / scale,
                            ]);

                        frequency *= 2.0;
                        amplitude *= persistence;
                    }

                    let chamber_noise = worley.get([wx / 50.0, wy / 50.0, wz / 50.0]);

                    density -= chamber_noise * 0.5;

                    let height = perlin.get([wx * 0.03, wz * 0.03]) * 20.0 + 10.0;
                    let voxel = if wy > height || density < -0.3 {
                        Block::default_empty()
                    } else if wy > height - 4.0 {
                        Block::Dirt
                    } else if density > 0.3 {
                        Block::default_opaque()
                    } else {
                        let t = (density + 0.3) / 0.6;
                        Block::lerp(Block::default_empty(), Block::default_opaque(), t)
                    };

                    data.push(voxel);
                }
            }
        }

        Self(data.try_into().unwrap())
    }
}

impl<V: Voxel> Chunk<V> {
    pub fn get(&self, pos: [u8; 3]) -> &V {
        let index = Self::to_index(pos);
        &self.0[index]
    }

    pub fn get_mut(&mut self, pos: [u8; 3]) -> &mut V {
        let index = Self::to_index(pos);
        &mut self.0[index]
    }

    const WIDTH_SQ: usize = (CHUNK_WIDTH as usize) * (CHUNK_WIDTH as usize);
    fn to_index([x, y, z]: [u8; 3]) -> usize {
        debug_assert!(
            x < CHUNK_WIDTH && y < CHUNK_WIDTH && z < CHUNK_WIDTH,
            "Coordinates out of bounds"
        );
        x as usize + y as usize * (CHUNK_WIDTH as usize) + z as usize * Self::WIDTH_SQ
    }

    pub fn generate_mesh(
        &self,
        meshes: &mut ResMut<Assets<Mesh>>,
        neighbors: &[Option<&Chunk<V>>; 6],
    ) -> Handle<Mesh> {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let pos = [x, y, z];
                    let voxel = self.get(pos);
                    if !voxel.is_opaque() {
                        continue;
                    }

                    for face in Face::ALL {
                        self.add_face_if_visible(
                            pos,
                            face,
                            voxel,
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut indices,
                            neighbors,
                        );
                    }
                }
            }
        }

        // let normals: Vec<_> = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices));

        let handle = meshes.add(mesh);
        handle
    }

    fn add_face_if_visible(
        &self,
        pos: [u8; 3],
        face: Face,
        voxel: &V,
        positions: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        indices: &mut Vec<u32>,
        neighbors: &[Option<&Chunk<V>>; 6],
    ) {
        if self.cull_face(pos, face, neighbors) {
            return;
        }

        let [x, y, z] = pos.map(|v| v as f32);
        let base_index = positions.len() as u32;

        let unit_u = ((V::all().len() - 1) as f32).recip();
        let unit_v = 6f32.recip();

        let atlas_index = V::all().iter().position(|v| v == voxel).unwrap() - 1;
        let u = atlas_index as f32 * unit_u;
        let v;

        match face {
            Face::Left => {
                v = 0.0;
                positions.extend([
                    [x, y, z],
                    [x, y + 1.0, z],
                    [x, y + 1.0, z + 1.0],
                    [x, y, z + 1.0],
                ]);
                normals.extend([[1.0, 0.0, 0.0]; 4]);
            }
            Face::Right => {
                v = 3.0 * unit_v;
                positions.extend([
                    [x + 1.0, y, z],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z],
                ]);
                normals.extend([[-1.0, 0.0, 0.0]; 4]);
            }
            Face::Down => {
                v = 1.0 * unit_v;
                positions.extend([
                    [x, y, z],
                    [x, y, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y, z],
                ]);
                normals.extend([[0.0, 1.0, 0.0]; 4]);
            }
            Face::Up => {
                v = 4.0 * unit_v;
                positions.extend([
                    [x, y + 1.0, z],
                    [x + 1.0, y + 1.0, z],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x, y + 1.0, z + 1.0],
                ]);
                normals.extend([[0.0, -1.0, 0.0]; 4]);
            }
            Face::Front => {
                v = 2.0 * unit_v;
                positions.extend([
                    [x, y, z],
                    [x + 1.0, y, z],
                    [x + 1.0, y + 1.0, z],
                    [x, y + 1.0, z],
                ]);
                normals.extend([[0.0, 0.0, 1.0]; 4]);
            }
            Face::Back => {
                v = 5.0 * unit_v;
                positions.extend([
                    [x, y, z + 1.0],
                    [x, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                ]);
                normals.extend([[0.0, 0.0, -1.0]; 4]);
            }
        }

        uvs.extend([
            [u, v],
            [u + unit_u, v],
            [u + unit_u, v + unit_v],
            [u, v + unit_v],
        ]);

        indices.extend([
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    fn cull_face(&self, pos: [u8; 3], face: Face, neighbors: &[Option<&Chunk<V>>; 6]) -> bool {
        let [x, y, z] = pos;
        match face {
            Face::Left => {
                if x == 0 {
                    neighbors[Face::Left as usize]
                        .is_some_and(|c| c.get([CHUNK_WIDTH - 1, y, z]).is_opaque())
                } else {
                    self.get([x - 1, y, z]).is_opaque()
                }
            }
            Face::Right => {
                if x == CHUNK_WIDTH - 1 {
                    neighbors[Face::Right as usize].is_some_and(|c| c.get([0, y, z]).is_opaque())
                } else {
                    self.get([x + 1, y, z]).is_opaque()
                }
            }
            Face::Down => {
                if y == 0 {
                    neighbors[Face::Down as usize]
                        .is_some_and(|c| c.get([x, CHUNK_WIDTH - 1, z]).is_opaque())
                } else {
                    self.get([x, y - 1, z]).is_opaque()
                }
            }
            Face::Up => {
                if y == CHUNK_WIDTH - 1 {
                    neighbors[Face::Up as usize].is_some_and(|c| c.get([x, 0, z]).is_opaque())
                } else {
                    self.get([x, y + 1, z]).is_opaque()
                }
            }
            Face::Front => {
                if z == 0 {
                    neighbors[Face::Front as usize]
                        .is_some_and(|c| c.get([x, y, CHUNK_WIDTH - 1]).is_opaque())
                } else {
                    self.get([x, y, z - 1]).is_opaque()
                }
            }
            Face::Back => {
                if z == CHUNK_WIDTH - 1 {
                    neighbors[Face::Back as usize].is_some_and(|c| c.get([x, y, 0]).is_opaque())
                } else {
                    self.get([x, y, z + 1]).is_opaque()
                }
            }
        }
    }

    pub fn to_rle(&self) -> Vec<(u16, V)> {
        let mut rle = Vec::new();
        let mut count = 0u16;
        let mut last = &self.0[0];

        for voxel in self.0.iter() {
            if voxel == last && count < u16::MAX {
                count += 1;
            } else {
                rle.push((count, *last));
                count = 1;
                last = voxel;
            }
        }
        rle.push((count, *last));
        rle
    }

    pub fn from_rle(rle: &[(u16, V)]) -> Self {
        let mut data = Vec::with_capacity(CHUNK_SIZE);
        for (count, value) in rle {
            data.extend(iter::repeat(value).take(*count as usize));
        }
        assert_eq!(data.len(), CHUNK_SIZE, "RLE data doesn't match chunk size");
        Self(data.try_into().unwrap())
    }
}
