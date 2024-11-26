use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_plugins(lan_shootmans::netcode::server_plugin)
        .run();
}