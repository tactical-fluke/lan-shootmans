use crate::cursor::CursorState;
use crate::health::{Health, PotentialDamageEvent};
use crate::lifetime::Lifetime;
use crate::{player_ui, GRAVITY};
use bevy::color::palettes::basic::RED;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_rapier3d::prelude::*;
use std::time::Duration;
use crate::dev_console::{AddConsoleVariable, DeveloperConsole};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct Noclip;

#[derive(Bundle)]
pub struct FirstPersonPlayerBundle {
    controller_marker: Player,
    spatial: SpatialBundle,
    character_controller: KinematicCharacterController,
    collider: Collider,
    health: Health
}

impl FirstPersonPlayerBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(transform),
            ..default()
        }
    }
}

impl Default for FirstPersonPlayerBundle {
    fn default() -> Self {
        Self {
            controller_marker: Player,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0., 0., 0.),
                ..Default::default()
            },
            collider: Collider::round_cylinder(0.9, 0.3, 0.2),
            character_controller: KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
            health: Health::with_max(100),
        }
    }
}

#[derive(Bundle)]
pub struct FirstPersonCameraBundle {
    marker: PlayerCamera,
    camera: Camera3dBundle,
}

// Should generally be made as a child of the FirstPersonPlayerBundle, so this entity is offset to the main body
impl Default for  FirstPersonCameraBundle {
    fn default() -> Self {
        Self {
            marker: PlayerCamera,
            camera: Camera3dBundle {
                transform: Transform::from_xyz(0., 0.2, -0.1),
                ..default()
            }
        }
    }
}

#[derive(Resource)]
pub struct PlayerData {
    speed: f32,
    sensitivity: f32,
    jump_speed: f32,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            speed: 20.0,
            sensitivity: 0.1,
            jump_speed: 10.0,
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct MovementInput(Vec3);

#[derive(Resource, Deref, DerefMut, Default)]
struct LookInput(Vec2);

fn handle_player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut movement_input: ResMut<MovementInput>,
    mut look_input: ResMut<LookInput>,
    mut mouse_events: EventReader<MouseMotion>,
    player_data: Res<PlayerData>,
) {
    let mut intended_movement = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyA) {
        intended_movement.x -= 1.0
    }
    if keys.pressed(KeyCode::KeyD) {
        intended_movement.x += 1.0
    }
    if keys.pressed(KeyCode::KeyW) {
        intended_movement.z -= 1.0
    }
    if keys.pressed(KeyCode::KeyS) {
        intended_movement.z += 1.0
    }

    **movement_input = intended_movement.normalize_or_zero();

    if keys.pressed(KeyCode::Space) {
        movement_input.y = 1.0;
    }
    if keys.pressed(KeyCode::ControlLeft) {
        movement_input.y = -1.0;
    }

    for event in mouse_events.read() {
        look_input.x -= event.delta.x * player_data.sensitivity;
        look_input.y -= event.delta.y * player_data.sensitivity;
        look_input.y = look_input.y.clamp(-89.9, 89.9); // Limit pitch
    }
}

fn handle_noclip_movement(
    mut player_query: Query<
        &mut Transform,
        (With<Player>, With<Noclip>),
    >,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    time: Res<Time>,
    mut movement_input: ResMut<MovementInput>,
    player_data: Res<PlayerData>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        error!("no player camera");
        return;
    };

    let Ok(mut transform) = player_query.get_single_mut() else {
        return;
    };

    let movement = Vec3::new(movement_input.x, 0., movement_input.z);
    let mut movement = camera_transform.compute_transform().rotation * movement * time.delta_seconds() * player_data.speed;
    movement += Vec3::new(0., movement_input.y, 0.) * player_data.speed * time.delta_seconds();
    **movement_input = Vec3::ZERO;

    transform.translation += movement;
}

fn handle_normal_player_movement(
    mut player_query: Query<
        (
            &Transform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        (With<Player>, Without<Noclip>)
    >,
    time: Res<Time>,
    mut movement_input: ResMut<MovementInput>,
    mut grounded_timer: Local<f32>,
    mut vertical_movement: Local<f32>,
    player_data: Res<PlayerData>,
) {
    let Ok((transform, mut controller, output)) = player_query.get_single_mut() else {
        return;
    };

    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(movement_input.x, 0.0, movement_input.z) * player_data.speed;
    let jump_speed = movement_input.y * player_data.jump_speed;
    // Clear input
    **movement_input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = 0.5;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
}

fn handle_player_look(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    look_input: Res<LookInput>,
    cursor_state: Res<CursorState>,
) {
    if **cursor_state == CursorGrabMode::None {
        return;
    }

    let Ok(mut transform) = player_query.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::Y, look_input.x.to_radians());
    let Ok(mut transform) = camera_query.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::X, look_input.y.to_radians());
}

pub fn create_player_at_location(commands: &mut Commands, position: Vec3) {
    commands
        .spawn(FirstPersonPlayerBundle::new(Transform::from_translation(position)))
        .with_children(|b| {
            b.spawn(FirstPersonCameraBundle::default());
        });
}

fn shootmans(
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    player_query: Query<Entity, With<Player>>,
    rapier_context: Res<RapierContext>,
    keys: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut damage_event_writer: EventWriter<PotentialDamageEvent>,
) {
    let Ok(player_camera_transform) = camera_query.get_single() else {
        error!("could not find camera");
        return;
    };

    let Ok(player) = player_query.get_single() else {
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

        if let Some((entity, toi)) = rapier_context.cast_ray(
            ray_pos,
            ray_direction.as_vec3(),
            max_toi,
            solid,
            query_filter,
        ) {
            let hit_point = ray_pos + (ray_direction * toi);
            debug!("Hit entity '{:?}' at pos {}", entity, hit_point);

            damage_event_writer.send(PotentialDamageEvent {
                entity,
                damage: 25,
                originating_entity: player,
            });

            #[cfg(debug_assertions)]
            commands
                .spawn(Lifetime(Timer::new(
                    Duration::from_secs(2),
                    TimerMode::Once,
                )))
                .insert(PbrBundle {
                    mesh: meshes.add(Sphere { radius: 0.25 }.mesh().uv(8, 4)),
                    material: materials.add(Color::NONE),
                    transform: Transform::from_translation(hit_point),
                    ..Default::default()
                })
                .insert(Wireframe)
                .insert(WireframeColor { color: RED.into() });
        }
    }
}

fn update_noclip(
    mut last_noclip_value: Local<bool>,
    console: Res<DeveloperConsole>,
    player: Query<Entity, With<Player>>,
    mut commands: Commands
) {
    if let Ok(noclip) = console.get_value::<bool>("noclip") {
        if *last_noclip_value != noclip {
            let entity = player.get_single().unwrap();
            if noclip {
                commands.entity(entity).remove::<Collider>()
                    .insert(Noclip);
            } else {
                commands.entity(entity).insert(Collider::round_cylinder(0.9, 0.3, 0.2))
                    .remove::<Noclip>();
            }
            *last_noclip_value = noclip;
        }
    }
}

pub fn first_person_controller_plugin(app: &mut App) {
    app.insert_resource(PlayerData::default())
        .insert_resource(MovementInput::default())
        .insert_resource(LookInput::default())
        .add_systems(PreUpdate, handle_player_input)
        .add_systems(FixedUpdate, (
            handle_normal_player_movement,
            handle_noclip_movement
        ))
        .add_systems(Update, handle_player_look)
        .add_systems(Update, shootmans)
        .add_cvar("noclip".into(), false)
        .add_systems(Update, update_noclip)
        .add_plugins(player_ui::PlayerUiPlugin);
}
