use std::{
    array,
    f32::consts::PI,
    fs::File,
    io::{BufReader, BufWriter},
    marker::PhantomData,
};

use crate::{
    block::Block,
    chunk::{
        Chunk, ChunkDirty, ChunkMeshUpdateRequest, ChunkNeighbors, ChunkNeighborsUpdateRequest,
        ChunkPos, CHUNK_WIDTH,
    },
    player::Player,
    voxel::Voxel,
};

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    utils::hashbrown::HashSet,
};
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct Seed(pub u32);

pub struct WorldPlugin<V: Voxel>(pub PhantomData<V>);

impl<V: Voxel> Plugin for WorldPlugin<V> {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin)
            .insert_resource(Seed(rand::random_range(0..1000000)))
            .add_systems(
                Update,
                (
                    unload_chunks::<V>,
                    load_local_chunks,
                    update_chunk_neighbors::<V>,
                    update_chunk_meshes::<V>,
                )
                    .chain(),
            );
    }
}

#[derive(thiserror::Error, Debug)]
enum WorldError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Serialize, Deserialize, Reflect)]
struct WorldSave<V> {
    chunks: Vec<(ChunkPos, Vec<(u16, V)>)>,
}

impl<V: Voxel> Default for WorldSave<V> {
    fn default() -> Self {
        Self { chunks: Vec::new() }
    }
}

impl<V: Voxel> WorldSave<V> {
    fn new() -> Self {
        Self::default()
    }
}

impl<V: Voxel> WorldSave<V> {
    pub fn from_chunks(
        chunks: &mut Query<(Entity, &ChunkPos, &Chunk<V>), With<ChunkDirty>>,
        commands: &mut Commands,
    ) -> Self {
        Self {
            chunks: chunks
                .iter()
                .map(|(entity, &pos, chunk)| {
                    commands.entity(entity).remove::<ChunkDirty>();
                    (pos, chunk.to_rle())
                })
                .collect(),
        }
    }
    pub fn save(&self, path: &str) -> Result<(), WorldError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, WorldError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let save: WorldSave<V> = bincode::deserialize_from(reader)?;
        Ok(save)
    }
}

pub fn update_chunk_neighbors<V: Voxel>(
    mut commands: Commands,
    requested_chunks: Query<(Entity, &ChunkPos), With<ChunkNeighborsUpdateRequest>>,
    all_chunks: Query<(Entity, &ChunkPos)>,
) {
    for (entity, pos) in requested_chunks.iter() {
        let neighbors = pos.offsets().map(|offset| {
            all_chunks
                .iter()
                .find_map(|(ne, npos)| (entity != ne && npos.0 == offset.0).then_some(ne))
        });

        commands
            .entity(entity)
            .insert(ChunkNeighbors(neighbors))
            .remove::<ChunkNeighborsUpdateRequest>();
    }
}

pub fn update_chunk_meshes<V: Voxel>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    requested_chunks: Query<(Entity, &ChunkNeighbors, &Chunk<V>), With<ChunkMeshUpdateRequest>>,
    all_chunks: Query<&Chunk<V>>,
) {
    for (entity, neighbors, chunk) in requested_chunks.iter() {
        let neighbor_refs = array::from_fn(|i| {
            neighbors.0[i].and_then(|neighbor_entity| all_chunks.get(neighbor_entity).ok())
        });

        let mesh_handle = chunk.generate_mesh(&mut meshes, &neighbor_refs);

        commands
            .entity(entity)
            .insert(Mesh3d(mesh_handle))
            .remove::<ChunkMeshUpdateRequest>();
    }
}

pub const RENDER_DISTANCE: u8 = 8;

pub fn load_local_chunks(
    seed: Res<Seed>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Query<&Transform, With<Player>>,
    chunks: Query<&ChunkPos, With<Chunk<Block>>>,
) {
    let player_transform = player.single();
    let player_pos = player_transform.translation / CHUNK_WIDTH as f32;
    let player_chunk = ChunkPos(player_pos.to_array().map(|x| x.floor() as i32));
    let ChunkPos([px, py, pz]) = player_chunk;

    let world = WorldSave::<Block>::load("assets/world.bin").unwrap();

    let existing_chunks: HashSet<_> = chunks.iter().map(|pos| *pos).collect();

    let rdf = RENDER_DISTANCE as f32;

    let mut chunks_to_load = Vec::with_capacity((4.0 / 3.0 * PI * rdf.powi(3)).ceil() as usize);
    let rdi = RENDER_DISTANCE as i32;
    for dx in -rdi..=rdi {
        for dy in -rdi..=rdi {
            for dz in -rdi..=rdi {
                if (dx * dx + dy * dy + dz * dz) as f32 <= rdf.powi(2) {
                    chunks_to_load.push(ChunkPos([px + dx, py + dy, pz + dz]));
                }
            }
        }
    }

    for pos in chunks_to_load {
        let posf = Vec3::from_array(pos.0.map(|x| x as f32));

        if let Some((_, rle)) = world.chunks.iter().find(|(p, _)| *p == pos) {
            let chunk = Chunk::from_rle(&rle[..]);
            let mesh = chunk.generate_mesh(&mut meshes, &[None; 6]);
            commands.spawn((
                chunk,
                pos,
                Mesh3d(mesh),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: Some(asset_server.load("textures/atlas.png")),
                    perceptual_roughness: 0.8,
                    ..Default::default()
                })),
                // Wireframe,
                Transform::from_translation(posf * CHUNK_WIDTH as f32),
                ChunkMeshUpdateRequest,
                ChunkNeighborsUpdateRequest,
            ));
        } else if !existing_chunks.contains(&pos) {
            let chunk = Chunk::<Block>::generate(pos, seed.0);
            let mesh = chunk.generate_mesh(&mut meshes, &[None; 6]);
            commands.spawn((
                chunk,
                pos,
                Mesh3d(mesh),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: Some(asset_server.load("textures/atlas.png")),
                    perceptual_roughness: 0.8,
                    ..Default::default()
                })),
                // Wireframe,
                Transform::from_translation(posf * CHUNK_WIDTH as f32),
                ChunkMeshUpdateRequest,
                ChunkNeighborsUpdateRequest,
                ChunkDirty,
            ));
        }
    }
}

pub fn unload_chunks<V: Voxel>(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    chunks: Query<(Entity, &ChunkPos), With<Chunk<V>>>,
) {
    let player_transform = player.single();
    let player_pos = player_transform.translation / CHUNK_WIDTH as f32;
    let player_chunk = ChunkPos(player_pos.to_array().map(|x| x.floor() as i32));
    let ChunkPos([px, py, pz]) = player_chunk;

    let rdf = RENDER_DISTANCE as f32;
    for (entity, _) in chunks.iter().filter(|(_, ChunkPos([cx, cy, cz]))| {
        let dx = px - cx;
        let dy = py - cy;
        let dz = pz - cz;
        (dx * dx + dy * dy + dz * dz) as f32 > rdf.powi(2)
    }) {
        commands.entity(entity).despawn();
    }
}
