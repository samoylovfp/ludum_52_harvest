use super::*;

#[derive(Component)]
pub struct StartMarker;

pub struct Start;

impl Plugin for Start {
    fn build(&self, app: &mut App) {
        // app
        //     .add_system_set(
        //         SystemSet::on_enter(AppState::Start)
        //         .with_system(spawn_start));
    }
}
