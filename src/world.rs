use bevy::prelude::*;
use serde_json::Value;

use crate::data::Chunk;

pub struct WorldPlugin;

#[derive(Resource)]
pub struct RenderedChunks(Vec<Chunk>);

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    let world_data: Value = serde_json::from_str(include_str!("../assets/worlds/main/world.json"))
        .expect("Invalid world data!");
}
