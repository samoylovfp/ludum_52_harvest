use crate::{
    buggy::{buggy_movement_and_control, setup_buggy},
    harvester::{add_harvester, move_harvesters},
    util::image_from_aseprite,
    AppState, PIXEL_MULTIPLIER,
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use once_cell::sync::OnceCell;

#[derive(Component)]
pub struct TerrainMarker;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_exit(AppState::Start)
                .with_system(setup_terrain)
                .with_system(setup_buggy),
        )
        .add_system_set(SystemSet::on_enter(AppState::Terrain).with_system(enable_terrain_cam))
        .add_system(move_harvesters)
        .add_system(buggy_movement_and_control)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(12.0))
        .add_plugin(RapierDebugRenderPlugin::default());
    }
}

fn setup_terrain(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut phys: ResMut<RapierConfiguration>,
) {
    static TERRAIN_IMAGE_CELL: OnceCell<Image> = OnceCell::new();
    static TERRAIN_TEXTURE_HANDLE_CELL: OnceCell<Handle<Image>> = OnceCell::new();

    let terrain_image = TERRAIN_IMAGE_CELL.get_or_init(|| {
        image_from_aseprite(
            include_bytes!("../assets/placeholders/terrain.aseprite"),
            "Background",
        )
    });
    let terrain_texture_handle =
        TERRAIN_TEXTURE_HANDLE_CELL.get_or_init(|| textures.add(terrain_image.clone()));
    let size = terrain_image.size() * PIXEL_MULTIPLIER;

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: terrain_texture_handle.clone(),
            ..default()
        })
        .insert(TerrainMarker);
    commands.spawn((Camera2dBundle::default(), TerrainMarker));
    phys.gravity = Vec2 { x: 0.0, y: 0.0 };

    // FIXME remove this when proper harvester spawning is implemented
    add_harvester(commands, textures, (0, 0), 0);
}

fn enable_terrain_cam(
    mut cam: Query<&mut Camera, With<TerrainMarker>>,
    mut panel_cam: Query<&mut Camera, Without<TerrainMarker>>,
) {
    cam.for_each_mut(|mut c| c.is_active = true);
    panel_cam.for_each_mut(|mut c| c.is_active = false);
}
