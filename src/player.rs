use std::time::Duration;
use bevy::color::palettes::basic::RED;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use crate::{GRAVITY, player_ui};
use crate::health::{Health, PotentialDamageEvent};

#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub  struct PlayerCamera;

#[derive(Bundle)]
pub struct FirstPersonPlayerBundle {
    controller_marker: Player,
    physics_basis: RigidBody,
    character_controller: KinematicCharacterController,
    collider: Collider,
    spatial_bundle: SpatialBundle,
}

#[derive(Resource)]
pub struct PlayerData {
    speed: f32,
    sensitivity: f32,
    max_fall_speed: f32,
}

#[derive(Component, Default)]
struct CharacterControllerData {
    velocity: Vec3,
    grounded: bool,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            speed: 20.0,
            sensitivity: 0.0005,
            max_fall_speed: 100.0,
        }
    }
}

#[derive(Component)]
struct Lifetime(Timer);

fn handle_player_movement(
    mut player_query: Query<(&mut KinematicCharacterController, &mut CharacterControllerData, &Transform), With<Player>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_data: ResMut<PlayerData>,
    physics_config: Res<RapierConfiguration>
) {
    if let Ok((mut controller, mut data, transform)) = player_query.get_single_mut() {
        let forward = transform.forward();
        let backward = transform.back();
        let left = transform.left();
        let right = transform.right();

        let mut intended_movement = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyA) {
            intended_movement += left.as_vec3();
        }
        if keys.pressed(KeyCode::KeyD) {
            intended_movement += right.as_vec3();
        }
        if keys.pressed(KeyCode::KeyW) {
            intended_movement += forward.as_vec3();
        }
        if keys.pressed(KeyCode::KeyS) {
            intended_movement += backward.as_vec3();
        }

        let intended_movement = intended_movement.normalize_or_zero() * player_data.speed * time.delta_seconds();
        data.velocity.x = intended_movement.x;
        data.velocity.z = intended_movement.z;

        if !data.grounded {
            data.velocity += physics_config.gravity * time.delta_seconds();
            data.velocity.y = data.velocity.y.clamp(-player_data.max_fall_speed, player_data.max_fall_speed); //ASSUMPTION gravity is only in +/- Y
        } else {
            data.velocity.y = 0.0; // ASSUMPTION
        }
        if let Some(mut translation) = controller.translation {
            translation += data.velocity;
        }
        else {
            controller.translation = Some(data.velocity);
        }
    }
}

fn handle_player_look(
    mut player_query: Query<(&mut Transform, &Children), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_data: Res<PlayerData>,
    mut mouse_events: EventReader<MouseMotion>,
    primary_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = if let Ok(window) = primary_window.get_single() {
        window
    }
    else {
        error!("could not find primary window");
        return;
    };

    let window_scale = window.width().min(window.height());
    if let Ok((mut player_transform, children)) = player_query.get_single_mut() {
        match window.cursor.grab_mode {
            CursorGrabMode::None => {}
            _ => {
                let mut camera_transform = if let Ok(camera) = camera_query.get_mut(children[0]) {
                    camera
                } else {
                    error!("no player camera");
                    return;
                };

                let mut rotation = Vec2::ZERO;
                rotation.x = player_transform.rotation.to_euler(EulerRot::YXZ).0;
                rotation.y = camera_transform.rotation.to_euler(EulerRot::YXZ).1;
                for event in mouse_events.read() {
                    rotation.x -= (event.delta.x * player_data.sensitivity * window_scale).to_radians();
                    rotation.y -= (event.delta.y * player_data.sensitivity * window_scale).to_radians();
                    rotation.y = rotation.y.clamp(-1.54, 1.54); // clamp pitch
                }
                camera_transform.rotation = Quat::from_axis_angle(Vec3::X, rotation.y);
                player_transform.rotation = Quat::from_axis_angle(Vec3::Y, rotation.x);
            }
        }

    }
}

pub fn create_player_at_location(commands: &mut Commands, position: Vec3) {
    commands.
        spawn(FirstPersonPlayerBundle {
            controller_marker: Player,
            character_controller: KinematicCharacterController{
                offset: CharacterLength::Absolute(0.1),
                slide: true,
                snap_to_ground: Some(CharacterLength::Absolute(0.5)),
                ..Default::default()
            },
            collider: Collider::capsule(Vect{x:0.0, y:0.0, z:0.0}, Vect{x:0.0, y: 1.0, z:0.0}, 0.5),
            spatial_bundle: SpatialBundle::from_transform(Transform::from_xyz(position.x, position.y, position.z)),
            physics_basis: RigidBody::KinematicPositionBased,
        })
        .insert(Restitution::coefficient(0.7))
        .insert(CharacterControllerData::default())
        .insert(Health::with_max(100))
        .with_children(|parent| {
            parent
                .spawn(PlayerCamera)
                .insert(Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 0.9, 0.0),
                    ..Default::default()
                });
        });
}

fn debug_player(
    player_transform_query: Query<(Entity, &KinematicCharacterControllerOutput)>
) {
    for (entity_id, player) in player_transform_query.iter() {
        debug!("entity {:?} moving: {:?}, grounded: {:?}", entity_id, player.effective_translation, player.grounded);
    }
}

fn handle_grounded(
    mut controller_data: Query<(&mut CharacterControllerData, &KinematicCharacterControllerOutput)>
) {
    for (mut controller_data, output) in &mut controller_data {
        controller_data.grounded = output.grounded;
    }
}

fn shootmans(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    player_query: Query<Entity, With<Player>>,
    rapier_context: Res<RapierContext>,
    keys: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut damage_event_writer: EventWriter<PotentialDamageEvent>
) {
    if let Ok(player_camera_transform) = camera_query.get_single() {
        let player = if let Ok(player) = player_query.get_single() {
            player
        } else {
            error!("could not find player");
            return;
        };

        if keys.just_pressed(MouseButton::Left) {
            let ray_pos = player_camera_transform.translation();
            let ray_direction = player_camera_transform.forward();
            let max_toi = 500.0;
            let solid = true;
            let query_filter = QueryFilter {
                flags: Default::default(),
                groups: None,
                exclude_collider: Some(player),
                exclude_rigid_body: None,
                predicate: None,
            };

            if let Some((entity, toi)) = rapier_context.cast_ray(ray_pos, ray_direction.as_vec3(), max_toi, solid, query_filter) {
                let hit_point = ray_pos + (ray_direction * toi);
                debug!("Hit entity '{:?}' at pos {}", entity, hit_point);

                damage_event_writer.send(PotentialDamageEvent {
                    entity,
                    damage: 25,
                    originating_entity: player
                });

                #[cfg(debug_assertions)]
                commands
                    .spawn(Lifetime(Timer::new(Duration::from_secs(2), TimerMode::Once)))
                    .insert(PbrBundle {
                        mesh: meshes.add(Sphere { radius: 0.25 }.mesh().uv(8, 4)),
                        material: materials.add(Color::NONE),
                        transform: Transform::from_translation(hit_point),
                        ..Default::default()
                    })
                    .insert(Wireframe)
                    .insert(WireframeColor{ color: RED.into() });
            }
        }
    } else {
        error!("could not find camera");
        return;
    }
}

fn handle_lifetimes(
    mut lifetimes: Query<(&mut Lifetime, Entity)>,
    time: Res<Time>,
    mut commands: Commands
) {
    for (mut lifetime, entity) in lifetimes.iter_mut() {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn first_person_controller_plugin(app: &mut App) {
    app
        .insert_resource(PlayerData::default())
        .add_systems(Update,handle_player_movement)
        .add_systems(Update, handle_player_look)
        .add_systems(Update, debug_player.after(handle_player_movement))
        .add_systems(PostUpdate, handle_grounded)
        .add_systems(Update, shootmans)
        .add_systems(Update, handle_lifetimes)
        .add_plugins(player_ui::PlayerUiPlugin);
}
