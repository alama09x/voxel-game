mod camera;
mod data;
mod world;

use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    pbr::wireframe::WireframePlugin,
    prelude::*,
};
use camera::CameraPlugin;
use data::Chunk;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            CameraPlugin,
            WireframePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_gizmos)
        .run();
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X * 16.0, BLUE);
    gizmos.line(Vec3::ZERO, Vec3::Y * 16.0, GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z * 16.0, RED);
    for x in -8..8 {
        for z in -8..8 {
            for y in -4..4 {
                let pos = Vec3::new(
                    x as f32 * Chunk::WIDTH as f32,
                    y as f32 * Chunk::WIDTH as f32,
                    z as f32 * Chunk::WIDTH as f32,
                );
                gizmos.cuboid(
                    Transform::from_translation(pos).with_scale(Vec3::splat(Chunk::WIDTH as f32)),
                    Color::srgba(1.0, 0.0, 0.0, 0.1),
                )
            }
        }
    }
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn rectangular prism of chunks
    for x in -8..8 {
        for z in -8..8 {
            for y in -4..4 {
                let chunk = Chunk::new(1, [x, y, z]);
                commands.spawn((
                    Mesh3d(meshes.add(chunk)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load("textures/atlas.png")),
                        ..default()
                    })),
                    Transform::from_xyz(
                        x as f32 * Chunk::WIDTH as f32,
                        y as f32 * Chunk::WIDTH as f32,
                        z as f32 * Chunk::WIDTH as f32,
                    ),
                ));
            }
        }
    }
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-32.0, 32.0, 32.0),
    ));
}
