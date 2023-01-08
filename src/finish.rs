use bevy::text::Text2dBounds;

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
	mut fonts: ResMut<Assets<Font>>,
) {
    let picture;
	let text;
    if tanks.0 >= CANISTERS_TO_WIN {
        picture = img_handle_and_size_from_bytes(
            include_bytes!("../assets/spriteendgood.aseprite"),
            "Layer 1",
            &mut textures,
        );
		text = "Good this text wraPps in the box d  dhfhf dhdhd fhfhhf dfhfhf dhdh ksksks shshshhshs shs shd dhd d ddjjddnd djdjdjjdjd djdjdndj hh".to_uppercase();
    } else {
        picture = img_handle_and_size_from_bytes(
            include_bytes!("../assets/spriteendbad.aseprite"),
            "Layer 1",
            &mut textures,
        );
		text = "Bad this text wraPps in the box d  dhfhf dhdhd fhfhhf dfhfhf dhdh ksksks shshshhshs shs shd dhd d ddjjddnd djdjdjjdjd djdjdndj hh".to_uppercase();
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
	let font = include_bytes!("../assets/PublicPixel-z84yD.ttf");
    // FIXME (samoylovfp) deduplicate
    let font_handle = fonts.add(Font::try_from_bytes(font.to_vec()).expect("valid font"));


	let box_size = Vec2::new(600.0, 600.0);
    let box_position = Vec2::new(0.0, -250.0);
	let text_style = TextStyle {
        font: font_handle,
        font_size: 20.0,
        color: Color::WHITE,
    };

	commands.spawn(Text2dBundle {
        text: Text::from_section(text, text_style),
        text_2d_bounds: Text2dBounds {
            size: box_size,
        },

        transform: Transform::from_xyz(
            box_position.x - box_size.x / 2.0,
            box_position.y + box_size.y / 2.0,
            1.0,
        ),
        ..default()
    });
	
}
