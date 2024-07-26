mod flycam;
mod player;

use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

const GRAVITY: Vect = Vect{x:0.0, y: -9.8, z: 0.0 };

#[derive(Component)]
struct Player;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(flycam::flycam_plugin)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_player)
        .add_systems(Update, print_ball_altitude)
        .add_systems(Update, handle_gravity_for_kinematic_characters)
        .run();
}

fn setup_player(mut commands: Commands) {
    commands
        .spawn(Player)
        .insert(KinematicCharacterController {
           ..Default::default()
        })
        .insert(Collider::capsule(Vect{x: 0.0, y:-0.5, z: 0.0}, Vect{x: 0.0, y:0.5, z:0.0}, 0.25))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(0.0, 5.0, 0.0)));
}

fn setup_physics(mut commands: Commands, mut physics_config: ResMut<RapierConfiguration>) {

    physics_config.gravity = GRAVITY;

    commands
        .spawn(Collider::cuboid(100.0, 0.5, 100.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, 0.0)));

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 5.0, 0.0)));
}

fn print_ball_altitude(mut positions: Query<&mut Transform, With<RigidBody>>) {
    for mut transform in positions.iter_mut() {
        dbg!(transform.rotation.to_axis_angle());
        transform.rotation = Quat::from_rotation_z(270_f32.to_radians());
    }
}

fn handle_gravity_for_kinematic_characters(
    mut character_controllers: Query<&mut KinematicCharacterController>,
    time: Res<Time>
) {
    for mut controller in character_controllers.iter_mut() {
        if let Some(mut translation) = controller.translation {
            translation += GRAVITY * time.delta_seconds();
        } else {
            controller.translation = Some(GRAVITY * time.delta_seconds());
        }
    }
}


