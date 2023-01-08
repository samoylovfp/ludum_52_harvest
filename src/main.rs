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
    let start = || {
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
            .add_system(start_music)
            .insert_resource(MusicStarted(false))
            .run()
    };

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        console_error_panic_hook::set_once();
        let doc = web_sys::window().unwrap().document().unwrap();
        let mut button = doc
            .create_element("button")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        button.set_inner_html("Start");
        let button_clone = button.clone();
        doc.body().unwrap().append_child(&button).unwrap();
        let f = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            doc.body().unwrap().remove_child(&button_clone).unwrap();
            start()
        }) as Box<dyn FnMut()>);
        button.set_onclick(Some(f.as_ref().unchecked_ref()));

        f.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        start()
    }
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

#[derive(Resource)]
struct MusicStarted(bool);

fn start_music(
    mut commands: Commands,
    audio: Res<Audio>,
    mut source: ResMut<Assets<AudioSource>>,
    buttons: Res<Input<MouseButton>>,
    mut playing: ResMut<MusicStarted>,
) {
    if playing.0 {
        return;
    }
    if buttons.just_pressed(MouseButton::Left) {
        let data = StaticSoundData::from_cursor(
            Cursor::new(include_bytes!("../assets/theme.ogg")),
            StaticSoundSettings::default(),
        )
        .unwrap();
        let handle = source.add(AudioSource { sound: data });
        audio.play(handle).with_volume(0.2).looped();
        playing.0 = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Start,
    Terrain,
    Panel,
    Finish,
}
