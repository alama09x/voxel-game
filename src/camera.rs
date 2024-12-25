use std::{f32::consts::FRAC_PI_2, ops::Range};

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::CursorGrabMode};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRotationSettings>()
            .init_resource::<CameraMovementSettings>()
            .add_systems(Startup, camera_setup)
            .add_systems(Update, (rotate_camera, move_camera));
    }
}

#[derive(Debug, Resource)]
struct CameraMovementSettings {
    speed: f32,
}

impl Default for CameraMovementSettings {
    fn default() -> Self {
        Self { speed: 10.0 }
    }
}

#[derive(Debug, Resource)]
struct CameraRotationSettings {
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
}

impl Default for CameraRotationSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            pitch_range: -pitch_limit..pitch_limit,
            pitch_speed: 0.004,
            yaw_speed: 0.004,
        }
    }
}

fn camera_setup(mut commands: Commands, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;

    commands.spawn((
        Camera3d::default(),
        Transform::default().looking_at(Vec3::X + Vec3::Z, Vec3::Y),
    ));
}

fn rotate_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    settings: Res<CameraRotationSettings>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    let delta = mouse_motion.delta;
    if delta != Vec2::ZERO {
        let delta_yaw = delta.x * settings.yaw_speed;
        let delta_pitch = delta.y * settings.pitch_speed;

        let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);

        let yaw = yaw - delta_yaw;
        let pitch =
            (pitch - delta_pitch).clamp(settings.pitch_range.start, settings.pitch_range.end);

        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

fn move_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraMovementSettings>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    let speed_factor = settings.speed * time.delta_secs();
    let xz_mask = Vec3::X + Vec3::Z;
    let local_x = (camera.local_x().as_vec3() * xz_mask).normalize() * speed_factor;
    let local_z = (camera.local_z().as_vec3() * xz_mask).normalize() * speed_factor;

    for key in keys.get_pressed() {
        match key {
            KeyCode::KeyW => camera.translation -= local_z,
            KeyCode::KeyA => camera.translation -= local_x,
            KeyCode::KeyS => camera.translation += local_z,
            KeyCode::KeyD => camera.translation += local_x,
            KeyCode::Space => camera.translation.y += speed_factor,
            KeyCode::ShiftLeft => camera.translation.y -= speed_factor,
            _ => (),
        }
    }
}
