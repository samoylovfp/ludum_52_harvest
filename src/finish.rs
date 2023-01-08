use crate::{panel::PanelMarker, terrain::TerrainMarker};

use super::*;

#[derive(Component)]
pub struct FinishMarker;

pub struct Finish;

impl Plugin for Finish {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Finish).with_system(despawn_everything));
		app.add_system_set(SystemSet::on_exit(AppState::Finish).with_system(despawn_really_everything));
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

fn despawn_really_everything(
    mut commands: Commands,
    entities: Query<Entity>,
) {
    entities.for_each(|e| commands.entity(e).despawn());
}
