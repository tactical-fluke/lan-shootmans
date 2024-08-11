use bevy::prelude::*;

#[derive(Component)]
pub struct Health{
    pub max: i32,
    pub current: i32,
}

#[derive(Event)]
pub struct PotentialDamageEvent {
    pub entity: Entity,
    pub originating_entity: Entity,
    pub damage: i32,
}

impl Health {
    pub fn with_max(max: i32) -> Self {
        Self {
            max,
            current: max
        }
    }
}

fn handle_damage_events(
    mut damage_events: EventReader<PotentialDamageEvent>,
    mut health_query: Query<&mut Health>,
    mut commands: Commands
) {
    for event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(event.entity) {
            health.current -= event.damage;

            if health.current <= 0 {
                commands.entity(event.entity).despawn();
            }
        }
    }
}

pub fn health_plugin(app: &mut App) {
    app
        .add_event::<PotentialDamageEvent>()
        .add_systems(Update, handle_damage_events);
}
