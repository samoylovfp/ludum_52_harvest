use std::io::Cursor;

use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
use harvester::update_center;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use tooltip::update_tooltip;
use util::load_assets;

mod buggy;
mod finish;
mod harvester;
mod panel;
mod start;
mod terrain;
mod tooltip;
mod util;

pub const PIXEL_MULTIPLIER: f32 = 5.0;
pub const CELL_SIZE_TERRAIN: f32 = 40.0;
pub const WIDTH: f32 = 160.0 * PIXEL_MULTIPLIER;
pub const HEIGHT: f32 = 120.0 * PIXEL_MULTIPLIER;

fn main() {
    App::new()
        .add_state(AppState::Start)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Moon 2023".to_string(),
                width: WIDTH,
                height: HEIGHT,
                ..default()
            },
            ..default()
        }))
        .add_plugin(start::StartPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .add_plugin(panel::PanelPlugin)
        .add_plugin(finish::Finish)
        .add_plugin(AudioPlugin)
        // .add_startup_system(spawn_tooltip)
        .add_system(handle_input)
        .add_system(update_tooltip)
        .add_system(update_center)
        .add_startup_system(load_assets)
        .add_startup_system(music)
        .run();
}

fn handle_input(keys: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        let state = app_state.current().clone();
        app_state
            .set(match state {
                AppState::Start => AppState::Terrain,
                AppState::Terrain => AppState::Panel,
                AppState::Panel => AppState::Terrain,
                AppState::Finish => AppState::Start,
            })
            .unwrap()
    }
}

fn music(audio: Res<Audio>, mut source: ResMut<Assets<AudioSource>>) {
    let data = StaticSoundData::from_cursor(
        Cursor::new(include_bytes!("../assets/theme.ogg")),
        StaticSoundSettings::default(),
    )
    .unwrap();
    let handle = source.add(AudioSource { sound: data });
    audio.play(handle).with_volume(0.2).looped();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Start,
    Terrain,
    Panel,
    Finish,
}
