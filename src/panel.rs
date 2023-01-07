use super::*;

#[derive(Component)]
pub struct PanelMarker;

pub struct Panel;

impl Plugin for Panel {
    fn build(&self, app: &mut App) {
        // app
        //     .add_system_set(
        //         SystemSet::on_enter(AppState::Panel)
        //         .with_system(spawn_start));
    }
}
