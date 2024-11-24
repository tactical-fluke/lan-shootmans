use std::collections::VecDeque;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Key, ScrollArea, TextEdit};

pub trait DeveloperConsoleValue: Sized {
    fn dev_console_parse(source: &str) -> Result<Self, String>;
    fn console_to_string(&self) -> String;
}

macro_rules! simple_dev_console_value {
    ($type:ty) => {
        impl DeveloperConsoleValue for $type {
            fn dev_console_parse(source: &str) -> Result<Self, String> {
                source.parse::<$type>().map_err(|_| format!("Failed to parse {} from {}", stringify!($ty), source))
            }

            fn console_to_string(&self) -> String { self.to_string() }
        }
    };
}

impl DeveloperConsoleValue for String {
    fn dev_console_parse(source: &str) -> Result<Self, String> {
        Ok(source.to_owned())
    }

    fn console_to_string(&self) -> String {
        self.clone()
    }
}

simple_dev_console_value!(bool);
simple_dev_console_value!(u8);
simple_dev_console_value!(u16);
simple_dev_console_value!(u32);
simple_dev_console_value!(u64);
simple_dev_console_value!(u128);
simple_dev_console_value!(i8);
simple_dev_console_value!(i16);
simple_dev_console_value!(i32);
simple_dev_console_value!(i64);
simple_dev_console_value!(i128);
simple_dev_console_value!(f32);
simple_dev_console_value!(f64);

enum DevConsoleLineSource {
    System,
    User
}

impl DevConsoleLineSource {
    pub fn get_line_symbol(&self) -> &str {
        match self {
            DevConsoleLineSource::System => ">",
            DevConsoleLineSource::User => "$",
        }
    }
}

#[derive(Resource)]
pub struct DeveloperConsole {
    values: HashMap<String, String>,
    lines: VecDeque<(DevConsoleLineSource, String)>,
    history_allowed: usize,
    show: bool,
    buf: String
}

impl DeveloperConsole {
    pub fn new(history_allowed: usize) -> DeveloperConsole {
        Self {
            values: HashMap::new(),
            lines: VecDeque::with_capacity(history_allowed),
            history_allowed,
            show: false,
            buf: String::new()
        }
    }

    pub fn get_value<T: DeveloperConsoleValue>(&self, key: &str) -> Result<T, String> {
        let value_string = self.values.get(key).ok_or(format!("Unknown value {}", key))?;

        T::dev_console_parse(value_string.as_str())
    }

   pub fn get_value_or_insert<T: DeveloperConsoleValue>(&mut self, key: &str, value: T) -> Result<T, String> {
       T::dev_console_parse(self.values.entry(String::from(key)).or_insert_with(|| value.console_to_string()))
   }

    pub fn set_value<T: DeveloperConsoleValue>(&mut self, key: &str, value: T) {
        self.values.insert(key.to_string(), value.console_to_string());
    }

    fn push_line(&mut self, line: (DevConsoleLineSource, String)) {
        self.lines.push_back(line);
        if self.lines.len() > self.history_allowed {
            self.lines.pop_front();
        }
    }
}

fn listen_for_dev_console_enable(
    keys: Res<ButtonInput<KeyCode>>,
    mut developer_console: ResMut<DeveloperConsole>
) {
    if keys.just_pressed(KeyCode::Backquote) {
        developer_console.show = !developer_console.show;
    }
}

fn dev_console_ui(mut ctx: EguiContexts, mut developer_console: ResMut<DeveloperConsole>) {
    if developer_console.show && ctx.try_ctx_mut().is_some() {
        egui::Window::new("Dev Console")
            .default_size([512., 512.])
            .title_bar(true)
            .resizable(true)
            .show(ctx.ctx_mut(), |ui| {
                ui.vertical(|ui| {
                    let scroll_height = ui.available_height() - 30.0;

                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .max_height(scroll_height)
                        .show(ui, |ui| {
                            for line in &developer_console.lines {
                                ui.label(format!("{}{}", line.0.get_line_symbol(), line.1));
                            }
                        });
                });

                ui.separator();

                let text_edit = TextEdit::singleline(&mut developer_console.buf)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .font(egui::TextStyle::Monospace);

                let text_edit_response = ui.add(text_edit);

                if text_edit_response.lost_focus() && ui.input(|ui| ui.key_pressed(Key::Enter)) {
                    if developer_console.buf.is_empty() {
                        developer_console.push_line((DevConsoleLineSource::User, String::new()));
                    } else {
                        let command = developer_console.buf.trim().to_string();
                        developer_console.push_line((DevConsoleLineSource::User, command.clone()));
                        let tokens = command.split(" ").collect::<Vec<&str>>();
                        if tokens.len() < 2 {
                            developer_console.push_line((DevConsoleLineSource::System, format!("Incorrect command: {:?}", tokens)));
                            return;
                        }
                        developer_console.set_value(tokens[0], tokens[1].to_string());
                        developer_console.buf.clear();
                    }
                }
        });
    }
}

pub trait AddConsoleVariable {
    fn add_cvar<T: DeveloperConsoleValue + Copy + Send + Sync + 'static>(&mut self, key: &'static str, value: T) -> &mut Self;
}

impl AddConsoleVariable for App {
    fn add_cvar<T: DeveloperConsoleValue + Copy + Send + Sync + 'static>(&mut self, key: &'static str, value: T) -> &mut Self {
        let system = move |mut developer_console: ResMut<DeveloperConsole>| {
            developer_console.set_value(key, value);
        };

        self.add_systems(Startup, system)
    }
}

pub fn developer_console_plugin(app: &mut App) {
    app
        .insert_resource(DeveloperConsole::new(40))
        .add_systems(Update, (listen_for_dev_console_enable, dev_console_ui));
}
