use bevy::app::{App, FixedUpdate, Startup, Update};
use bevy::ecs::event::ManualEventReader;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::log::error;
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::prelude::{Camera, Camera3dBundle, Commands, Component, Events, KeyCode, Query, Res, ResMut, Resource, Time, Transform, Window, With};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::math::Vect;

#[derive(Component)]
struct FlyCam;

#[derive(Resource)]
struct FlyCamSettings {
    speed: f32,
    sensitivity: f32,
}

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

impl Default for FlyCamSettings {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.001,
        }
    }
}

fn handle_fly_cam(
    mut cameras: Query<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<FlyCamSettings>
) {
    for mut camera_transform in &mut cameras {
        let mut translation = Vect::ZERO;
        let forward = camera_transform.forward();
        let backward = camera_transform.back();
        let left = camera_transform.left();
        let right = camera_transform.right();

        if keys.pressed(KeyCode::KeyA) {
            translation += left.as_vec3();
        }
        if keys.pressed(KeyCode::KeyD) {
            translation += right.as_vec3();
        }
        if keys.pressed(KeyCode::KeyW) {
            translation += forward.as_vec3();
        }
        if keys.pressed(KeyCode::KeyS) {
            translation += backward.as_vec3();
        }
        if keys.pressed(KeyCode::Space) {
            translation += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            translation -= Vec3::Y;
        }

        translation = translation.normalize_or_zero();
        camera_transform.translation += translation * time.delta_seconds() * settings.speed;
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn grab_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
        }
    }
    else {
        error!("primary window could not be found");
    }
}

fn fly_camera_look(
    mut fly_cam: Query<&mut Transform, With<Camera>>,
    camera_settings: Res<FlyCamSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in fly_cam.iter_mut() {
            for event in state.reader_motion.read(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        pitch -= (camera_settings.sensitivity * event.delta.y * window_scale).to_radians();
                        yaw -= (camera_settings.sensitivity * event.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    }
    else {
        error!("could not find primary window");
    }
}


fn setup_flycam(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0),
        ..Default::default()
    })
        .insert(FlyCam);
}

pub fn flycam_plugin(app: &mut App) {
    app
        .insert_resource(FlyCamSettings::default())
        .insert_resource(InputState::default())
        .add_systems(Startup, setup_flycam)
        .add_systems(FixedUpdate, handle_fly_cam)
        .add_systems(Update, grab_cursor)
        .add_systems(FixedUpdate, fly_camera_look);

}