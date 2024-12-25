mod camera;
mod data;

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
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk = Chunk::new();
    commands.spawn((
        Mesh3d(meshes.add(chunk)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/atlas.png")),
            ..default()
        })),
        Transform::default(),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-32.0, 32.0, 32.0),
    ));
}
