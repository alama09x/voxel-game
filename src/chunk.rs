use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::{NoiseFn, Perlin};

use crate::voxel::{Voxel, VOXEL_SIZE};

pub const CHUNK_SIZE: usize = 32;
pub const SEA_LEVEL: isize = 32;

#[derive(Bundle)]
pub struct ChunkBundle {
    chunk: Chunk,
    pbr_bundle: PbrBundle,
}

impl ChunkBundle {
    pub fn new(
        chunk: Chunk,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        Self {
            chunk,
            pbr_bundle: PbrBundle {
                mesh: meshes.add(chunk.into()),
                material: materials.add(chunk.into()),
                transform: Transform::from_xyz(
                    chunk.chunk_x as f32 * CHUNK_SIZE as f32,
                    chunk.chunk_y as f32 * CHUNK_SIZE as f32,
                    chunk.chunk_z as f32 * CHUNK_SIZE as f32,
                ),
                ..default()
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Chunk {
    pub voxels: [[[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub padding: ChunkPadding,
    pub chunk_x: isize,
    pub chunk_y: isize,
    pub chunk_z: isize,
    pub entity: Option<Entity>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkPadding {
    pub left: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
    pub right: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
    pub below: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
    pub above: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
    pub behind: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
    pub front: [[Option<Voxel>; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new(seed: u32, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> Self {
        let perlin = Perlin::new(seed);
        let in_range = |x: isize| x >= 0 && x < CHUNK_SIZE as isize;

        let mut voxels = [[[None; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let mut padding = ChunkPadding::default();
        for i in -1..=CHUNK_SIZE as isize {
            for k in -1..=CHUNK_SIZE as isize {
                let noise_x = (i as f64 + chunk_x as f64 * CHUNK_SIZE as f64) * 0.01;
                let noise_z = (k as f64 + chunk_z as f64 * CHUNK_SIZE as f64) * 0.01;

                let max_y = SEA_LEVEL + (perlin.get([noise_x, noise_z]) * 100.0).round() as isize;
                for j in -1..=CHUNK_SIZE as isize {
                    let y = j + chunk_y * CHUNK_SIZE as isize;
                    if y <= max_y {
                        let new_voxel = Voxel::default();
                        if in_range(i) && in_range(j) && in_range(k) {
                            voxels[i as usize][j as usize][k as usize] = Some(new_voxel);
                        } else if i == -1 && in_range(j) && in_range(k) {
                            padding.left[j as usize][k as usize] = Some(new_voxel);
                        } else if i == CHUNK_SIZE as isize && in_range(j) && in_range(k) {
                            padding.right[j as usize][k as usize] = Some(new_voxel);
                        } else if j == -1 && in_range(i) && in_range(k) {
                            padding.below[i as usize][k as usize] = Some(new_voxel);
                        } else if j == CHUNK_SIZE as isize && in_range(i) && in_range(k) {
                            padding.above[i as usize][k as usize] = Some(new_voxel);
                        } else if k == -1 && in_range(i) && in_range(j) {
                            padding.behind[i as usize][j as usize] = Some(new_voxel);
                        } else if k == CHUNK_SIZE as isize && in_range(i) && in_range(j) {
                            padding.front[i as usize][j as usize] = Some(new_voxel);
                        }
                    }
                }
            }
        }

        Self {
            voxels,
            padding,
            chunk_x,
            chunk_y,
            chunk_z,
            entity: None,
        }
    }

    pub fn calculate_delta(&self, other: Vec3) -> Vec3 {
        let chunk_pos = Vec3::new(
            self.chunk_x as f32 * VOXEL_SIZE * CHUNK_SIZE as f32,
            self.chunk_y as f32 * VOXEL_SIZE * CHUNK_SIZE as f32,
            self.chunk_z as f32 * VOXEL_SIZE * CHUNK_SIZE as f32,
        );
        return (other - chunk_pos).abs();
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl From<Chunk> for Mesh {
    fn from(chunk: Chunk) -> Self {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count = 0u32;

        for i in 0..CHUNK_SIZE {
            let x = i as f32 * VOXEL_SIZE - CHUNK_SIZE as f32 * VOXEL_SIZE * 0.5;
            let neg_x = x - VOXEL_SIZE * 0.5;
            let pos_x = x + VOXEL_SIZE * 0.5;

            for j in 0..CHUNK_SIZE {
                let y = j as f32 * VOXEL_SIZE - CHUNK_SIZE as f32 * VOXEL_SIZE * 0.5;
                let neg_y = y - VOXEL_SIZE * 0.5;
                let pos_y = y + VOXEL_SIZE * 0.5;

                for k in 0..CHUNK_SIZE {
                    if chunk.voxels[i][j][k].is_none() {
                        continue;
                    }

                    let z = k as f32 * VOXEL_SIZE - CHUNK_SIZE as f32 * VOXEL_SIZE * 0.5;
                    let neg_z = z - VOXEL_SIZE * 0.5;
                    let pos_z = z + VOXEL_SIZE * 0.5;

                    if (i > 0 && chunk.voxels[i - 1][j][k].is_none())
                        || (i == 0 && chunk.padding.left[j][k].is_none())
                    {
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

                    if (i < CHUNK_SIZE - 1 && chunk.voxels[i + 1][j][k].is_none())
                        || (i == CHUNK_SIZE - 1 && chunk.padding.right[j][k].is_none())
                    {
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

                    if (j > 0 && chunk.voxels[i][j - 1][k].is_none())
                        || (j == 0 && chunk.padding.below[i][k].is_none())
                    {
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

                    if (j < CHUNK_SIZE - 1 && chunk.voxels[i][j + 1][k].is_none())
                        || (j == CHUNK_SIZE - 1 && chunk.padding.above[i][k].is_none())
                    {
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

                    if (k > 0 && chunk.voxels[i][j][k - 1].is_none())
                        || (k == 0 && chunk.padding.behind[i][j].is_none())
                    {
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

                    if (k < CHUNK_SIZE - 1 && chunk.voxels[i][j][k + 1].is_none())
                        || (k == CHUNK_SIZE - 1 && chunk.padding.front[i][j].is_none())
                    {
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
        Self::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Self::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Self::ATTRIBUTE_NORMAL, normals)
            .with_indices(Some(Indices::U32(indices)))
    }
}

impl From<Chunk> for StandardMaterial {
    fn from(_chunk: Chunk) -> Self {
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
