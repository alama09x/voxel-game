use std::{f32::consts::FRAC_PI_2, ops::Range};

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::CursorGrabMode};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRotationSettings>()
            .init_resource::<PlayerMovementSettings>()
            .add_systems(Startup, setup)
            .add_systems(Update, (rotate_player, move_player));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Debug, Resource)]
struct PlayerMovementSettings {
    speed: f32,
}

impl Default for PlayerMovementSettings {
    fn default() -> Self {
        Self { speed: 10.0 }
    }
}

#[derive(Debug, Resource)]
struct PlayerRotationSettings {
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
}

impl Default for PlayerRotationSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            pitch_range: -pitch_limit..pitch_limit,
            pitch_speed: 0.004,
            yaw_speed: 0.004,
        }
    }
}

fn setup(mut commands: Commands, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;

    commands.spawn((
        Player,
        Camera3d::default(),
        Transform::default().looking_at(Vec3::X + Vec3::Z, Vec3::Y),
    ));
}

fn rotate_player(
    mouse_motion: Res<AccumulatedMouseMotion>,
    settings: Res<PlayerRotationSettings>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let delta = mouse_motion.delta;
    if delta != Vec2::ZERO {
        let delta_yaw = delta.x * settings.yaw_speed;
        let delta_pitch = delta.y * settings.pitch_speed;

        let (yaw, pitch, _) = player.rotation.to_euler(EulerRot::YXZ);

        let yaw = yaw - delta_yaw;
        let pitch =
            (pitch - delta_pitch).clamp(settings.pitch_range.start, settings.pitch_range.end);

        player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<PlayerMovementSettings>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let speed_factor = settings.speed * time.delta_secs();
    let xz_mask = Vec3::X + Vec3::Z;
    let local_x = (player.local_x().as_vec3() * xz_mask).normalize() * speed_factor;
    let local_z = (player.local_z().as_vec3() * xz_mask).normalize() * speed_factor;

    for key in keys.get_pressed() {
        match key {
            KeyCode::KeyW => player.translation -= local_z,
            KeyCode::KeyA => player.translation -= local_x,
            KeyCode::KeyS => player.translation += local_z,
            KeyCode::KeyD => player.translation += local_x,
            KeyCode::Space => player.translation.y += speed_factor,
            KeyCode::ShiftLeft => player.translation.y -= speed_factor,
            _ => (),
        }
    }
}
