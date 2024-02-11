use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::{NoiseFn, Perlin};

use crate::voxel::{Voxel, VOXEL_SIZE};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_PADDED: usize = 34;
pub const SEA_LEVEL: isize = 32;

#[derive(Component, Clone, Debug)]
pub struct Chunk {
    pub voxel_map: HashMap<[isize; 3], Voxel>,
    pub chunk_x: isize,
    pub chunk_y: isize,
    pub chunk_z: isize,
    pub entity: Option<Entity>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl Chunk {
    pub fn new(seed: u32, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> Self {
        let perlin = Perlin::new(seed);

        let mut voxel_map = HashMap::new();
        for x in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
            let noise_x = (x as f64 + chunk_x as f64 * CHUNK_SIZE as f64) * 0.01;
            for z in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
                let noise_z = (z as f64 + chunk_z as f64 * CHUNK_SIZE as f64) * 0.01;

                let max_y = SEA_LEVEL + (perlin.get([noise_x, noise_z]) * 100.0).round() as isize;
                for y in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
                    let world_y = y + chunk_y * CHUNK_SIZE as isize;
                    if world_y <= max_y {
                        let new_voxel = Voxel::default();
                        voxel_map.insert([x, y, z], new_voxel);
                    }
                }
            }
        }

        Self {
            voxel_map,
            chunk_x,
            chunk_y,
            chunk_z,
            entity: None,
        }
    }

    pub fn to_mesh(&self) -> Mesh {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count = 0u32;

        for x in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
            let neg_x = x as f32 - VOXEL_SIZE * 0.5;
            let pos_x = x as f32 + VOXEL_SIZE * 0.5;

            for y in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
                let neg_y = y as f32 - VOXEL_SIZE * 0.5;
                let pos_y = y as f32 + VOXEL_SIZE * 0.5;

                for z in -(CHUNK_SIZE_PADDED as isize / 2)..CHUNK_SIZE_PADDED as isize / 2 {
                    let neg_z = z as f32 - VOXEL_SIZE * 0.5;
                    let pos_z = z as f32 + VOXEL_SIZE * 0.5;

                    if self.voxel_map.get(&[x, y, z]).is_none()
                        || x.min(y.min(z)) == -(CHUNK_SIZE_PADDED as isize / 2)
                        || x.max(y.max(z)) == CHUNK_SIZE_PADDED as isize / 2 - 1
                    {
                        continue;
                    }

                    if self.voxel_map.get(&[x - 1, y, z]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [neg_x, neg_y, neg_z],
                                normal: [-1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [neg_x, neg_y, pos_z],
                                normal: [-1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [neg_x, pos_y, pos_z],
                                normal: [-1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [neg_x, pos_y, neg_z],
                                normal: [-1.0, 0.0, 0.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }

                    if self.voxel_map.get(&[x + 1, y, z]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [pos_x, neg_y, pos_z],
                                normal: [1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, neg_y, neg_z],
                                normal: [1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, neg_z],
                                normal: [1.0, 0.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, pos_z],
                                normal: [1.0, 0.0, 0.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }

                    if self.voxel_map.get(&[x, y - 1, z]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [neg_x, neg_y, neg_z],
                                normal: [0.0, -1.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, neg_y, neg_z],
                                normal: [0.0, -1.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, neg_y, pos_z],
                                normal: [0.0, -1.0, 0.0],
                            },
                            Vertex {
                                position: [neg_x, neg_y, pos_z],
                                normal: [0.0, -1.0, 0.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }

                    if self.voxel_map.get(&[x, y + 1, z]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [neg_x, pos_y, pos_z],
                                normal: [0.0, 1.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, pos_z],
                                normal: [0.0, 1.0, 0.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, neg_z],
                                normal: [0.0, 1.0, 0.0],
                            },
                            Vertex {
                                position: [neg_x, pos_y, neg_z],
                                normal: [0.0, 1.0, 0.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }

                    if self.voxel_map.get(&[x, y, z - 1]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [pos_x, neg_y, neg_z],
                                normal: [0.0, 0.0, -1.0],
                            },
                            Vertex {
                                position: [neg_x, neg_y, neg_z],
                                normal: [0.0, 0.0, -1.0],
                            },
                            Vertex {
                                position: [neg_x, pos_y, neg_z],
                                normal: [0.0, 0.0, -1.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, neg_z],
                                normal: [0.0, 0.0, -1.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }

                    if self.voxel_map.get(&[x, y, z + 1]).is_none() {
                        vertices.extend(&[
                            Vertex {
                                position: [neg_x, neg_y, pos_z],
                                normal: [0.0, 0.0, 1.0],
                            },
                            Vertex {
                                position: [pos_x, neg_y, pos_z],
                                normal: [0.0, 0.0, 1.0],
                            },
                            Vertex {
                                position: [pos_x, pos_y, pos_z],
                                normal: [0.0, 0.0, 1.0],
                            },
                            Vertex {
                                position: [neg_x, pos_y, pos_z],
                                normal: [0.0, 0.0, 1.0],
                            },
                        ]);
                        indices.extend(&[
                            vertex_count,
                            vertex_count + 1,
                            vertex_count + 2,
                            vertex_count,
                            vertex_count + 2,
                            vertex_count + 3,
                        ]);
                        vertex_count += 4;
                    }
                }
            }
        }

        let positions = vertices.iter().map(|v| v.position).collect::<Vec<_>>();
        let normals = vertices.iter().map(|v| v.normal).collect::<Vec<_>>();
        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_indices(Some(Indices::U32(indices)))
    }

    pub fn to_material(&self) -> StandardMaterial {
        Color::NONE.into()
    }
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.chunk_x == other.chunk_x
            && self.chunk_y == other.chunk_y
            && self.chunk_z == other.chunk_z
    }
}
