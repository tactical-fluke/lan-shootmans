use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max: i32,
    pub current: i32,
}

#[derive(Event)]
pub struct PotentialDamageEvent {
    pub originating_entity: Entity,
    pub damage: i32,
}

#[derive(Event)]
pub struct DeathEvent {
    originating_entity: Entity,
}

impl Health {
    pub fn with_max(max: i32) -> Self {
        Self { max, current: max }
    }
}

fn handle_damage_events(
    trigger: Trigger<PotentialDamageEvent>,
    mut health_query: Query<&mut Health>,
    mut commands: Commands,
) {
    if let Ok(mut health) = health_query.get_mut(trigger.entity()) {
        health.current -= trigger.event().damage;

        if health.current <= 0 {
            commands.trigger_targets(DeathEvent { originating_entity: trigger.event().originating_entity }, trigger.entity());
        }
    }
}

pub fn health_plugin(app: &mut App) {
    app.add_event::<PotentialDamageEvent>()
        .add_event::<DeathEvent>()
        .observe(handle_damage_events);
}
