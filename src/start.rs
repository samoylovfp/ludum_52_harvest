use std::time::Duration;

use bevy::prelude::*;

use crate::{terrain::TerrainMarker, AppState};

#[derive(Component)]
pub struct StartMarker;

pub struct StartPlugin;

impl Plugin for StartPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Start).with_system(spawn_start));
        app.add_system_set(SystemSet::on_exit(AppState::Start).with_system(despawn_start));
    }
}

fn spawn_start(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    commands.spawn((Camera2dBundle::default(), StartMarker));

    let font = include_bytes!("../assets/PublicPixel-z84yD.ttf");
    // FIXME (samoylovfp) deduplicate
    let font_handle = fonts.add(Font::try_from_bytes(font.to_vec()).expect("valid font"));
    commands
        .spawn(
            TextBundle::from_section(
                "Moon\n2023".to_uppercase(),
                TextStyle {
                    font: font_handle,
                    font_size: 100.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(StartMarker);
}

fn despawn_start(mut commands: Commands, start_entities: Query<Entity, With<StartMarker>>) {
    start_entities.for_each(|e| commands.entity(e).despawn());
}

pub fn set_timer(mut commands: Commands, time: Res<Time>) {
    commands.spawn((
        EndTimer {
            timer: Timer::new(Duration::from_secs(120), TimerMode::Once),
        },
        TerrainMarker,
    ));
}

pub fn check_end(
    mut timer: Query<&mut EndTimer>,
    time: Res<Time>,
    mut app_state: ResMut<State<AppState>>,
) {
	if timer.is_empty() {
		return ;
	}
    let mut timer = timer.single_mut();
    timer.timer.tick(time.delta());
	// println!("{}", timer.timer.remaining_secs());
    if timer.timer.finished() {
        app_state.set(AppState::Finish).unwrap();
    }
}

#[derive(Component)]
pub struct EndTimer {
    pub timer: Timer,
}
