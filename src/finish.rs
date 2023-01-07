use super::*;

#[derive(Component)]
pub struct FinishMarker;

pub struct Finish;

impl Plugin for Finish {
    fn build(&self, app: &mut App) {
        // app
        //     .add_system_set(
        //         SystemSet::on_enter(AppState::Finish)
        //         .with_system(spawn_start));
    }
}