use lan_shootmans::cursor::cursor_plugin;
use lan_shootmans::health::health_plugin;
use lan_shootmans::lifetime::lifetime_plugin;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_trenchbroom::prelude::*;
use lan_shootmans::dev_console::developer_console_plugin;
use lan_shootmans::trenchbroom;
use lan_shootmans::player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: repeating_image_sampler(false),
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(TrenchBroomPlugin::new(trenchbroom::trenchbroom_config()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WireframePlugin)
        .add_plugins(lan_shootmans::netcode::client_plugin)
        .add_plugins(developer_console_plugin)
        .add_plugins(health_plugin)
        .add_plugins(lifetime_plugin)
        .add_plugins(cursor_plugin)
        //.add_plugins(flycam::flycam_plugin)
        .add_plugins(player::first_person_controller_plugin)
        .add_systems(Startup, spawn_test_map)
        .run();
}

fn spawn_test_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map: Handle<Map> = asset_server.load("maps/test_map.map");
    commands.spawn(MapBundle {
        map,
        ..default()
    });
}
