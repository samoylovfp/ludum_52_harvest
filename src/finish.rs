use crate::{
    harvester::StoredCanisters,
    panel::PanelMarker,
    terrain::{TerrainMarker, CANISTERS_TO_WIN},
    util::img_handle_and_size_from_bytes,
};

use super::*;

#[derive(Component)]
pub struct FinishMarker;

pub struct Finish;

impl Plugin for Finish {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Finish)
                .with_system(despawn_everything)
                .with_system(spawn_finish),
        );
        app.add_system_set(
            SystemSet::on_exit(AppState::Finish).with_system(despawn_really_everything),
        );
    }
}

fn despawn_everything(
    mut commands: Commands,
    terrain_entities: Query<Entity, With<TerrainMarker>>,
    panel_entities: Query<Entity, With<PanelMarker>>,
) {
    terrain_entities.for_each(|e| commands.entity(e).despawn());
    panel_entities.for_each(|e| commands.entity(e).despawn());
}

fn despawn_really_everything(mut commands: Commands, entities: Query<Entity>) {
    entities.for_each(|e| commands.entity(e).despawn());
}

fn spawn_finish(
    mut commands: Commands,
    tanks: Res<StoredCanisters>,
    mut textures: ResMut<Assets<Image>>,
) {
    let mut picture;
    if tanks.0 >= CANISTERS_TO_WIN {
        picture = img_handle_and_size_from_bytes(
            include_bytes!("../assets/spriteendgood.aseprite"),
            "Layer 1",
            &mut textures,
        );
    } else {
        picture = img_handle_and_size_from_bytes(
            include_bytes!("../assets/spriteendbad.aseprite"),
            "Layer 1",
            &mut textures,
        );
    }
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(picture.1),
            ..default()
        },
        texture: picture.0,
        ..default()
    });
    // img_handle_and_size_from_bytes(panel_bytes, "exitup", &mut textures),
}
