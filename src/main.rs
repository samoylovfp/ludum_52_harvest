use bevy::prelude::*;
use harvester::update_center;
use tooltip::{spawn_tooltip, update_tooltip};
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
        .add_startup_system(spawn_tooltip)
        .add_system(handle_input)
        .add_system(update_tooltip)
        .add_system(update_center)
        .add_startup_system(load_assets)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Start,
    Terrain,
    Panel,
    Finish,
}
