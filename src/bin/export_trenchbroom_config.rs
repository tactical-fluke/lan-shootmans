use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(AssetPlugin::default())
        .add_plugins(bevy_trenchbroom::TrenchBroomPlugin::new(lan_shootmans::trenchbroom::trenchbroom_config()))
        .add_systems(Startup, lan_shootmans::trenchbroom::write_trenchbroom_config)
        .run();
}