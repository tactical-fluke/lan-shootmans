use std::fs::create_dir;
use std::path::Path;
use bevy_trenchbroom::config::TrenchBroomConfig;
use bevy_trenchbroom::entity_definitions;
use bevy::prelude::*;
use bevy::log::error;
use crate::player::MakeEntityPlayer;

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: char = '\\';

#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR: char = '/';

pub fn trenchbroom_config() -> TrenchBroomConfig {
    TrenchBroomConfig::new("lan-shootmans")
        .entity_scale_expression("scale")
        .entity_definitions(entity_definitions!{
            /// World Entity
            Solid worldspawn {} |world, entity, view| {
                info!("spawning world");
                // This is the code to spawn the entity into the world, note that the TrenchBroomConfig resource is not available in this scope
                // If you need to access the TrenchBroomConfig, access it via view.tb_config
                view.spawn_brushes(world, entity, BrushSpawnSettings::new().smooth_by_default_angle().pbr_mesh().trimesh_collider());
                // Here, we also call smooth_by_default_angle(), which smooths the normals of connected surfaces curving less than a default threshold
            }

            // Some useful base classes
            Base angles {
                /// Pitch Yaw Roll (Y Z X)
                angles: Vec3,
            }
            Base target_name {
                /// Name
                targetname: "target_source", // A custom type for a FGD property
            }
            Base target {
                /// Target
                target: "target_destination",
            }
            Base parent {
                parent: "target_destination",
            }

            Point light {
                        color: Color,
                        intensity: f32,
            } |world, entity, view| {
                world.entity_mut(entity).insert(PointLightBundle {
                    point_light: PointLight {
                        color: view.get("color")?,
                        intensity: view.get("intensity")?,
                        shadows_enabled: true,
                        ..default()
                    },
                    ..default()
                });
            }

            Point player_spawn(size(-16 -16 -32, 16 16 32) color(0 255 0)) {} |world, entity, view| {
                world.entity_mut(entity)
                    .make_player(view.get_transform());
            }
        })
}

pub fn write_trenchbroom_config(config: Res<TrenchBroomConfig>) {
    let root_game_dir = std::env::var("TRENCHBROOM_INSTALL_DIR").or::<u8>(Ok("./trenchbroom_config".into())).unwrap();
    let root_game_dir = Path::new(&root_game_dir);
    let shootmans_dir = format!("games{PATH_SEPARATOR}lan-shootmans");
    let shootmans_dir = Path::new(&shootmans_dir);
    let shootmans_dir = root_game_dir.join(shootmans_dir);
    let _ = create_dir(&shootmans_dir); // path may already exist, we don't care if so.
    if let Err(err) = config.write_folder(shootmans_dir) {
        error!("Could not write TrenchBroom config: {err}");
    }
}