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
    fn surface_heightmap([x, z]: [f64; 2], seed: u32) -> f64 {
        let perlin = Perlin::new(seed);
        let simplex = Simplex::new(seed);
        let worley = Worley::new(seed);

        let warp_scale = 50.0;
        let warp_strength = 2.0;

        let xz_warped =
            [x, z].map(|w| w + warp_strength * simplex.get([x / warp_scale, z / warp_scale]));

        let mut height = 0.0;
        let mut frequency = 0.8;
        let mut amplitude = 1.0;

        let scale = 100.0;
        let persistence = 0.4;
        let octaves = 8;

        for noise in [
            Box::new(perlin) as Box<dyn NoiseFn<f64, 2>>,
            Box::new(simplex),
            Box::new(worley),
        ] {
            for _ in 0..octaves {
                height += amplitude * noise.get(xz_warped.map(|x| x * frequency / scale));
                frequency *= 2.0;
                amplitude *= persistence;
            }
        }

        height * 50.0
    }

    fn cave_depth_field([x, y, z]: [f64; 3], seed: u32) -> f64 {
        let perlin = Simplex::new(seed + 1);
        let simplex = Simplex::new(seed + 1);
        let worley = Simplex::new(seed + 1);

        let scale = 100.0;
        let octaves = 6;
        let persistence = 0.5;

        let mut density = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;

        let warp_scale = 50.0;
        let warp_strength = 2.0;

        let xyz_warped =
            [x, y, z].map(|w| w + warp_strength * simplex.get([x / warp_scale, z / warp_scale]));

        for noise in [
            Box::new(perlin) as Box<dyn NoiseFn<f64, 3>>,
            Box::new(simplex),
            Box::new(worley),
        ] {
            for _ in 0..octaves {
                density += amplitude * noise.get(xyz_warped.map(|x| x * frequency / scale));
                frequency *= 2.0;
                amplitude *= persistence;
            }
        }

        density
    }

    pub fn generate(pos: ChunkPos, seed: u32) -> Self {
        let mut data = Vec::with_capacity(CHUNK_SIZE);
        let [cx, cy, cz] = pos.0;

        // World coordinates
        let cw = CHUNK_WIDTH as f64;
        let base_x = cx as f64 * cw;
        let base_y = cy as f64 * cw;
        let base_z = cz as f64 * cw;

        // Equation for voxel indices is x(cw)^2 + y(cw) + z
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_WIDTH {
                    let wx = base_x + x as f64;
                    let wy = base_y + y as f64;
                    let wz = base_z + z as f64;

                    let height = Self::surface_heightmap([wx, wz], seed);
                    let density = Self::cave_depth_field([wx, wy, wz], seed);

                    let block = if wy < height - 16.0 {
                        if density < -0.3 {
                            Block::Air
                        } else if density > 0.3 {
                            Block::Air
                        } else {
                            let t = (density + 0.3) / 0.6;
                            Block::lerp(Block::Air, Block::Stone, t)
                        }
                    } else if wy < height - 8.0 {
                        Block::Stone
                    } else if wy < height - 4.0 {
                        Block::Dirt
                    } else if wy < height {
                        Block::Grass
                    } else {
                        Block::Air
                    };

                    data.push(block);
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
        x as usize * Self::WIDTH_SQ + y as usize * (CHUNK_WIDTH as usize) + z as usize
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

        // for face in Face::ALL {
        //     self.greedy_quads(
        //         face,
        //         neighbors,
        //         &mut positions,
        //         &mut normals,
        //         &mut uvs,
        //         &mut indices,
        //     );
        // }

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
                            neighbors,
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut indices,
                        );
                    }
                }
            }
        }

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
        neighbors: &[Option<&Chunk<V>>; 6],
        positions: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        indices: &mut Vec<u32>,
    ) {
        if self.cull_face(pos, face, neighbors) {
            return;
        }

        let [x, y, z] = pos.map(|v| v as f32);
        let base_index = positions.len() as u32;

        positions.extend(face.positions([x, y, z], [x + 1.0, y + 1.0, z + 1.0]));
        normals.extend([face.normal(); 4]);

        let unit_u = ((V::all().len() - 1) as f32).recip();
        let unit_v = 6f32.recip();

        let atlas_index = V::all().iter().position(|v| v == voxel).unwrap();
        let u0 = atlas_index as f32 * unit_u;
        let v0 = match face {
            Face::Left => 0.0,
            Face::Bottom => unit_v,
            Face::Back => unit_v * 2.0,
            Face::Right => unit_v * 3.0,
            Face::Top => unit_v * 4.0,
            Face::Front => unit_v * 5.0,
        };

        let u1 = u0 + unit_u;
        let v1 = v0 + unit_v;

        uvs.extend([[u0, v1], [u0, v0], [u1, v0], [u1, v1]]);

        indices.extend([
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    fn greedy_quads(
        &self,
        face: Face,
        neighbors: &[Option<&Chunk<V>>; 6],
        positions: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        indices: &mut Vec<u32>,
    ) {
        let mut mask = [[false; CHUNK_WIDTH as usize]; CHUNK_WIDTH as usize];

        for c in 0..CHUNK_WIDTH {
            for b in 0..CHUNK_WIDTH {
                for a in 0..CHUNK_WIDTH {
                    if mask[b as usize][a as usize] {
                        continue;
                    }

                    let pos = [a, b, c];
                    if !self.cull_face(pos, face, neighbors) {
                        let block = self.get(pos);
                        let mut width = 1;
                        let mut height = 1;

                        while a + width < CHUNK_WIDTH {
                            let next_pos = [a + width, b, c];
                            if self.cull_face([a + width, b, c], face, neighbors)
                                || self.get(next_pos) != block
                                || mask[b as usize][(a + width) as usize]
                            {
                                break;
                            }
                            width += 1;
                        }

                        'outer: while b + height < CHUNK_WIDTH {
                            for w in 0..width {
                                if self.cull_face([a + w, c, b + height], face, neighbors)
                                    || mask[(b + height) as usize][(a + w) as usize]
                                {
                                    break 'outer;
                                }
                            }
                            height += 1;
                        }

                        for h in 0..height {
                            for w in 0..width {
                                mask[(b + h) as usize][(a + w) as usize] = true;
                            }
                        }

                        let af = a as f32;
                        let bf = b as f32;
                        let cf = c as f32;
                        let wf = width as f32;
                        let hf = height as f32;

                        let (min, max) = match face {
                            Face::Left | Face::Right => {
                                ([cf, af, bf], [cf + 1.0, af + wf, bf + hf])
                            }
                            Face::Bottom | Face::Top => {
                                ([af, cf, bf], [af + wf, cf + 1.0, bf + hf])
                            }
                            Face::Back | Face::Front => {
                                ([af, bf, cf], [af + wf, bf + hf, cf + 1.0])
                            }
                        };

                        positions.extend(face.positions(min, max));
                        normals.extend([face.normal(); 4]);

                        let base_index = positions.len() as u32 - 4;
                        indices.extend([
                            base_index,
                            base_index + 1,
                            base_index + 2,
                            base_index,
                            base_index + 2,
                            base_index + 3,
                        ]);

                        let unit_u = ((V::all().len() - 1) as f32).recip();
                        let unit_v = 6f32.recip();

                        let atlas_index = V::all().iter().position(|v| v == block).unwrap();
                        let u0 = atlas_index as f32 * unit_u;
                        let v0 = match face {
                            Face::Left => 0.0,
                            Face::Bottom => unit_v,
                            Face::Back => unit_v * 2.0,
                            Face::Right => unit_v * 3.0,
                            Face::Top => unit_v * 4.0,
                            Face::Front => unit_v * 5.0,
                        };

                        let u1 = u0 + unit_u;
                        let v1 = v0 + unit_v;

                        uvs.extend([[u0, v1], [u0, v0], [u1, v0], [u1, v1]]);
                    }
                }
            }
        }
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
            Face::Bottom => {
                if y == 0 {
                    neighbors[Face::Bottom as usize]
                        .is_some_and(|c| c.get([x, CHUNK_WIDTH - 1, z]).is_opaque())
                } else {
                    self.get([x, y - 1, z]).is_opaque()
                }
            }
            Face::Top => {
                if y == CHUNK_WIDTH - 1 {
                    neighbors[Face::Top as usize].is_some_and(|c| c.get([x, 0, z]).is_opaque())
                } else {
                    self.get([x, y + 1, z]).is_opaque()
                }
            }
            Face::Back => {
                if z == 0 {
                    neighbors[Face::Back as usize]
                        .is_some_and(|c| c.get([x, y, CHUNK_WIDTH - 1]).is_opaque())
                } else {
                    self.get([x, y, z - 1]).is_opaque()
                }
            }
            Face::Front => {
                if z == CHUNK_WIDTH - 1 {
                    neighbors[Face::Front as usize].is_some_and(|c| c.get([x, y, 0]).is_opaque())
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
