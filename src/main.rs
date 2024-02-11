use bevy::{pbr::wireframe::WireframePlugin, prelude::*};

mod chunk;
mod player;
mod terrain;
mod voxel;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WireframePlugin,
            player::PlayerPlugin,
            terrain::TerrainPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup() {}
