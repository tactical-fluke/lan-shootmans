mod cursor;
mod flycam;
mod health;
mod lifetime;
mod player;
mod player_ui;

use crate::cursor::cursor_plugin;
use crate::health::{health_plugin, Health};
use crate::lifetime::lifetime_plugin;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_rapier3d::prelude::*;

const GRAVITY: Vect = Vect {
    x: 0.0,
    y: -9.8,
    z: 0.0,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WireframePlugin)
        .add_plugins(health_plugin)
        .add_plugins(lifetime_plugin)
        .add_plugins(cursor_plugin)
        //.add_plugins(flycam::flycam_plugin)
        .add_plugins(player::first_person_controller_plugin)
        .add_systems(Startup, setup_world)
        .add_systems(Startup, setup_player)
        .run();
}

fn setup_player(mut commands: Commands) {
    player::create_player_at_location(
        &mut commands,
        Vec3 {
            x: -10.0,
            y: 10.0,
            z: 10.0,
        },
    );
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..Default::default()
    });

    // floor
    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.1, 0.0),
            mesh: meshes.add(Cuboid {
                half_size: Vec3 {
                    x: 100.0,
                    y: 0.05,
                    z: 100.0,
                },
            }),
            material: debug_material.clone(),
            ..Default::default()
        });

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(Friction::coefficient(14.0))
        .insert(ColliderMassProperties::Density(2.0))
        .insert(Damping {
            linear_damping: 0.0,
            angular_damping: 1.0,
        })
        .insert(PbrBundle {
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            mesh: meshes.add(Sphere::mesh(&Default::default()).uv(32, 18)),
            material: debug_material.clone(),
            ..Default::default()
        })
        .insert(Health::with_max(100));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
