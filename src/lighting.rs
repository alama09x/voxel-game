use bevy::prelude::*;

use crate::{chunk::CHUNK_WIDTH, player::Player};

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(AmbientLight {
                brightness: 100.0,
                ..default()
            })
            .add_systems(PostStartup, setup);
    }
}

fn setup(mut commands: Commands, player: Query<Entity, With<Player>>) {
    commands.entity(player.single()).insert(PointLight {
        range: CHUNK_WIDTH as f32 * 4.0,
        intensity: 8192.0,
        shadows_enabled: true,
        ..default()
    });
}
