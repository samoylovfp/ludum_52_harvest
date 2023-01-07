use crate::util::image_from_aseprite;

use super::*;

#[derive(Component)]
pub struct TerrainMarker;

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(AppState::Terrain)
                .with_system(setup_terrain));
    }
}

fn setup_terrain(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    commands.spawn(SpriteBundle {
        texture: textures.add(image_from_aseprite(include_bytes!(
            "../assets/placeholders/terrain.aseprite"
        ))),
        ..default()
    });
}