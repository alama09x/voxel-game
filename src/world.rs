use std::{
    array,
    collections::VecDeque,
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
    player::PlayerMoveChunkEvent,
    voxel::Voxel,
};

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    utils::hashbrown::HashSet,
};
use serde::{Deserialize, Serialize};

pub const RENDER_DISTANCE: u8 = 8;
pub const CHUNKS_PER_FRAME: u8 = 8;

#[derive(Resource)]
pub struct Seed(pub u32);

pub struct WorldPlugin<V: Voxel>(pub PhantomData<V>);

impl<V: Voxel> Plugin for WorldPlugin<V> {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin)
            .insert_resource(Seed(rand::random_range(0..1000000)))
            .insert_resource(WorldSave::<Block>::load("assets/world.bin").unwrap_or_default())
            .init_resource::<ChunkManager>()
            .add_systems(
                Update,
                (
                    update_chunk_manager,
                    load_local_chunks::<V>,
                    update_chunk_neighbors::<V>,
                    update_chunk_meshes::<V>,
                )
                    .chain(),
            );
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WorldError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(Resource, Serialize, Deserialize, Reflect)]
pub struct WorldSave<V> {
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

#[derive(Resource, Default)]
pub struct ChunkManager {
    loaded_chunks: HashSet<ChunkPos>,
    load_queue: VecDeque<ChunkPos>,
    unload_queue: VecDeque<Entity>,
}

pub fn update_chunk_manager(
    mut player_move_chunk_event: EventReader<PlayerMoveChunkEvent>,
    chunks: Query<(Entity, &ChunkPos), With<Chunk<Block>>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for ev in player_move_chunk_event.read() {
        let ChunkPos([px, py, pz]) = ev.0;

        chunk_manager.loaded_chunks.clear();
        for (_, pos) in chunks.iter() {
            chunk_manager.loaded_chunks.insert(*pos);
        }

        let rdf = RENDER_DISTANCE as f32;
        let rdi = RENDER_DISTANCE as i32;
        let mut desired_chunks =
            HashSet::with_capacity((4.0 / 3.0 * PI * rdf.powi(3)).ceil() as usize);
        for dx in -rdi..=rdi {
            for dy in -rdi..=rdi {
                for dz in -rdi..=rdi {
                    let pos = ChunkPos([px + dx, py + dy, pz + dz]);
                    if (dx * dx + dy * dy + dz * dz) as f32 <= rdf.powi(2) {
                        desired_chunks.insert(pos);
                    }
                }
            }
        }

        for pos in desired_chunks.difference(&chunk_manager.loaded_chunks.clone()) {
            if !chunk_manager.load_queue.contains(pos) {
                chunk_manager.load_queue.push_back(*pos);
            }
        }

        for (entity, pos) in chunks.iter() {
            if !desired_chunks.contains(pos) && !chunk_manager.unload_queue.contains(&entity) {
                chunk_manager.unload_queue.push_back(entity);
            }
        }
    }
}

pub fn load_local_chunks<V: Voxel>(
    seed: Res<Seed>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_manager: ResMut<ChunkManager>,
    world: Res<WorldSave<V>>,
    chunks: Query<(Entity, &ChunkPos), With<Chunk<V>>>,
) {
    let mut entities_to_unload = Vec::new();

    for _ in 0..CHUNKS_PER_FRAME {
        if let Some(pos) = chunk_manager.load_queue.pop_front() {
            if chunk_manager.loaded_chunks.contains(&pos) {
                continue;
            }

            let posf = Vec3::from_array(pos.0.map(|x| x as f32));

            if let Some((_, rle)) = world.chunks.iter().find(|(p, _)| *p == pos) {
                let chunk = Chunk::from_rle(&rle[..]);
                commands.spawn((
                    chunk,
                    pos,
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(asset_server.load("textures/atlas.png")),
                        perceptual_roughness: 0.8,
                        ..Default::default()
                    })),
                    Wireframe,
                    Transform::from_translation(posf * CHUNK_WIDTH as f32),
                    ChunkNeighborsUpdateRequest,
                    ChunkMeshUpdateRequest,
                ));
            } else {
                let chunk = Chunk::<Block>::generate(pos, seed.0);
                commands.spawn((
                    chunk,
                    pos,
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(asset_server.load("textures/atlas.png")),
                        perceptual_roughness: 0.8,
                        ..Default::default()
                    })),
                    Wireframe,
                    Transform::from_translation(posf * CHUNK_WIDTH as f32),
                    ChunkNeighborsUpdateRequest,
                    ChunkMeshUpdateRequest,
                    ChunkDirty,
                ));
            }
            chunk_manager.loaded_chunks.insert(pos);

            for neighbor_pos in pos.offsets() {
                if let Some((entity, _)) = chunks.iter().find(|(_, p)| **p == neighbor_pos) {
                    commands
                        .entity(entity)
                        .insert(ChunkNeighborsUpdateRequest)
                        .insert(ChunkMeshUpdateRequest);
                }
            }
        }
    }

    if let Some(entity) = chunk_manager.unload_queue.pop_front() {
        if chunks.get(entity).is_ok() {
            entities_to_unload.push(entity);
        }
    }

    for entity in entities_to_unload {
        commands.entity(entity).despawn();
    }
}
