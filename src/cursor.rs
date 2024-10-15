use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::log::error;
use bevy::prelude::{Deref, DerefMut, KeyCode, PreUpdate, Query, Res, ResMut, Resource, Update, Window, With};
use bevy::window::{CursorGrabMode, PrimaryWindow};

// Want to avoid grabbing the whole window whenever we wanna make sure the cursor is grabbed
#[derive(Resource, Deref, DerefMut, Default)]
pub struct CursorState(CursorGrabMode);

pub fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

pub fn grab_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_state: ResMut<CursorState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        error!("primary window could not be found");
        return;
    };
    
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(&mut window);
    }
    **cursor_state = window.cursor.grab_mode;
}

pub fn cursor_plugin(app: &mut App) {
    app.insert_resource(CursorState::default())
        .add_systems(Update, grab_cursor);
}
