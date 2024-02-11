use bevy::{pbr::wireframe::Wireframe, prelude::*};

use crate::{
    chunk::{Chunk, ChunkBundle, CHUNK_SIZE},
    player::Player,
    voxel::VOXEL_SIZE,
};

pub const RENDER_DISTANCE_CHUNKS: usize = 8;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Terrain::default())
            .add_systems(Startup, generate_chunks)
            .add_systems(Update, process_terrain);
    }
}

#[derive(Resource, Clone, Default)]
pub struct Terrain {
    pub chunks: Vec<Chunk>,
}

fn generate_chunks(mut terrain: ResMut<Terrain>) {
    for i in -16..=16 {
        for j in -4..=4 {
            for k in -16..=16 {
                terrain.chunks.push(Chunk::new(0, i, j, k));
            }
        }
    }
}

fn process_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain: ResMut<Terrain>,
    q_player: Query<&Transform, With<Player>>,
) {
    let t_player = q_player.single();
    for ref mut chunk in &mut terrain.chunks {
        let delta = chunk.calculate_delta(t_player.translation);
        let dist = (delta.x.powi(2) + delta.y.powi(2) + delta.z.powi(2)).sqrt();
        let rd = RENDER_DISTANCE_CHUNKS as f32 * CHUNK_SIZE as f32 * VOXEL_SIZE as f32;
        if chunk.entity.is_none() && dist < rd {
            chunk.entity = Some(
                commands
                    .spawn((
                        ChunkBundle::new(**chunk, &mut meshes, &mut materials),
                        Wireframe,
                    ))
                    .id(),
            );
        } else if chunk.entity.is_some() && dist > rd {
            if let Some(mut e_cmds) = commands.get_entity(chunk.entity.unwrap()) {
                chunk.entity = None;
                e_cmds.despawn();
            }
        }
    }
}
