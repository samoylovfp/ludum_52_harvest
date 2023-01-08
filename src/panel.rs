use once_cell::sync::OnceCell;

use crate::util::image_from_aseprite;

use super::*;

#[derive(Component)]
pub struct PanelMarker;

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(AppState::Start).with_system(spawn_panel))
            .add_system_set(SystemSet::on_enter(AppState::Panel).with_system(enable_panel_cam));
    }
}

fn spawn_panel(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    static PANEL_IMAGE_CELL: OnceCell<Image> = OnceCell::new();
    static PANEL_TEXTURE_HANDLE_CELL: OnceCell<Handle<Image>> = OnceCell::new();
    let panel_offset = Vec3 {
        x: 100_000.0,
        y: 100_000.0,
        z: 0.0,
    };

    let panel_image = PANEL_IMAGE_CELL.get_or_init(|| {
        image_from_aseprite(include_bytes!("../assets/spritepanel8.aseprite"), "main")
    });
    let panel_texture_handle =
        PANEL_TEXTURE_HANDLE_CELL.get_or_init(|| textures.add(panel_image.clone()));

    let size = panel_image.size() * PIXEL_MULTIPLIER;

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(size),
            ..default()
        },
        transform: Transform {
            translation: panel_offset,
            ..default()
        },
        texture: panel_texture_handle.clone(),
        ..default()
    });
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                is_active: false,
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: panel_offset.x,
                    y: panel_offset.y,
                    z: 100.0,
                },
                ..default()
            },
            ..default()
        },
        PanelMarker,
    ));
}

fn enable_panel_cam(
    mut panel_cam: Query<&mut Camera, With<PanelMarker>>,
    mut other_cams: Query<&mut Camera, Without<PanelMarker>>,
) {
    panel_cam.for_each_mut(|mut c| c.is_active = true);
    other_cams.for_each_mut(|mut c| c.is_active = false);
}
