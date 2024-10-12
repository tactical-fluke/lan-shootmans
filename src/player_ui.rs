use crate::health::Health;
use crate::player::Player;
use bevy::color::Color;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::*;

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_ui)
            .add_systems(Update, handle_player_health_bar);
    }
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct ActiveHealthBar;

pub fn spawn_player_ui(mut commands: Commands) {
    let health_container = NodeBundle {
        style: Style {
            left: Val::Percent(15.),
            bottom: Val::Percent(15.),
            width: Val::Percent(25.),
            height: Val::Percent(5.),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::srgb(1., 0., 0.).into(),
        ..default()
    };

    let active_health_bar = NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
        background_color: Color::srgb(0., 1., 0.).into(),
        z_index: ZIndex::Local(2),
        ..default()
    };

    let mut health_text = TextBundle::from_section(
        "100",
        TextStyle {
            font_size: 35.,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_text_justify(JustifyText::Left)
    .with_style(Style {
        position_type: PositionType::Absolute,
        left: Val::Percent(5.),
        bottom: Val::Percent(5.),
        ..default()
    });
    health_text.z_index = ZIndex::Local(3);

    let health_bar_container_entity = commands
        .spawn(PlayerHealthBar)
        .insert(health_container)
        .id();
    let active_health_bar_entity = commands
        .spawn(active_health_bar)
        .insert(ActiveHealthBar)
        .id();
    let health_text = commands.spawn(health_text).id();
    commands
        .entity(health_bar_container_entity)
        .push_children(&[active_health_bar_entity, health_text]);
}

pub fn handle_player_health_bar(
    player_query: Query<&Health, With<Player>>,
    health_bar_container_query: Query<&Children, With<PlayerHealthBar>>,
    mut health_text: Query<&mut Text>,
    mut style_query: Query<&mut Style, With<ActiveHealthBar>>,
) {
   let Ok(player_health) = player_query.get_single() else {
        error!("could not find player!");
        return;
    };

    let Ok(container_children) = health_bar_container_query.get_single() else {
        error!("could not find player health bar!");
        return;
    };

    for &child in container_children {
        if let Ok(mut active_health_bar) = style_query.get_mut(child) {
            active_health_bar.width =
                Val::Percent(player_health.current as f32 / player_health.max as f32 * 100.);
        }

        if let Ok(mut text) = health_text.get_mut(child) {
            text.sections[0].value = format!("{}", player_health.current);
        }
    }
}
