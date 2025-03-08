mod block;
mod chunk;
mod ecs;
mod face;
mod lighting;
mod player;
mod voxel;
mod world;

use std::marker::PhantomData;

use bevy::prelude::*;
use block::Block;
use lighting::LightingPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PlayerPlugin,
            LightingPlugin,
            WorldPlugin::<Block>(PhantomData),
        ))
        .run();
}
