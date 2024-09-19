use bevy::app::App;
use bevy::prelude::{
    Commands, Component, Deref, DerefMut, Entity, Query, Res, Time, Timer, Update,
};

#[derive(Component, Deref, DerefMut)]
pub struct Lifetime(pub Timer);

fn handle_lifetimes(
    mut lifetimes: Query<(&mut Lifetime, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut lifetime, entity) in lifetimes.iter_mut() {
        lifetime.tick(time.delta());
        if lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn lifetime_plugin(app: &mut App) {
    app.add_systems(Update, handle_lifetimes);
}
