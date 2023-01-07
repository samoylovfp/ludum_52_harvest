use bevy::prelude::*;

mod start;
mod terrain;
mod panel;
mod finish;
mod util;
mod harvester;

pub const PIXEL_MULTIPLIER: f32 = 5.0;
pub const CELL_SIZE_TERRAIN: f32 = 40.0;

fn main() {
    App::new()
		.add_state(AppState::Start)
		.insert_resource(ClearColor(Color::BLACK))
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			window: WindowDescriptor {
			title: "Moon 2023".to_string(),
			width: 160.0 * PIXEL_MULTIPLIER,
			height: 120.0 * PIXEL_MULTIPLIER,
			..default()
			},
			..default()
		}))
		.add_plugin(start::Start)
		.add_plugin(terrain::Terrain)
		.add_plugin(panel::Panel)
		.add_plugin(finish::Finish)
        .add_startup_system(setup)
		.add_system(handle_input)
        .run();
}

fn handle_input(keys: Res<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        match app_state.current() {
			AppState::Start => app_state.set(AppState::Terrain).unwrap(),
			AppState::Terrain => app_state.set(AppState::Panel).unwrap(),
			AppState::Panel => app_state.set(AppState::Finish).unwrap(),
			AppState::Finish => app_state.set(AppState::Start).unwrap(),
		}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
	Start,
    Terrain,
    Panel,
    Finish,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
