use bevy::{input::mouse::MouseMotion, prelude::*};

pub const PLAYER_SPEED: f32 = 20.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_player);
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    camera_bundle: Camera3dBundle,
    player: Player,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Player;

impl PlayerBundle {
    pub fn new() -> Self {
        Self {
            camera_bundle: Camera3dBundle {
                transform: Transform::from_xyz(-64.0, 64.0, -64.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            player: Player,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(PlayerBundle::new());
}

fn move_player(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut e_motion: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();

    let local_x = transform.local_x() * (Vec3::X + Vec3::Z);
    let local_z = transform.local_z() * (Vec3::X + Vec3::Z);

    for key in keys.get_pressed() {
        match key {
            KeyCode::W => transform.translation -= local_z * PLAYER_SPEED * time.delta_seconds(),
            KeyCode::A => transform.translation -= local_x * PLAYER_SPEED * time.delta_seconds(),
            KeyCode::S => transform.translation += local_z * PLAYER_SPEED * time.delta_seconds(),
            KeyCode::D => transform.translation += local_x * PLAYER_SPEED * time.delta_seconds(),
            KeyCode::Space => transform.translation.y += PLAYER_SPEED * time.delta_seconds(),
            KeyCode::ShiftLeft => transform.translation.y -= PLAYER_SPEED * time.delta_seconds(),
            _ => {}
        }
    }

    for ev in e_motion.read() {
        transform.rotate_y(-ev.delta.x * 0.005);
        transform.rotate_local_x(-ev.delta.y * 0.005);
    }
}
