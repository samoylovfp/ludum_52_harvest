use std::io::Cursor;

use crate::{
    harvester::{add_harvester, CenterIcon, SlotIcon, SlotNumber, TotalHarvesters},
    util::{
        bevy_image_from_ase_image, get_cursor_pos_in_world_coord, PanelAssetHandlers,
        TerrainAssetHandlers,
    },
};

use super::*;

pub const PANEL_OFFSET: Vec3 = Vec3 {
    x: 100_000.0,
    y: 100_000.0,
    z: 0.0,
};

#[derive(Component)]
pub struct PanelMarker;

pub struct PanelPlugin;

#[derive(Resource)]
struct PanelState {
    building_harvester: bool,
}

#[derive(Component)]
struct HarvesterBlueprint;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(AppState::Start).with_system(set_up_panel))
            .add_system_set(SystemSet::on_enter(AppState::Panel).with_system(enable_panel_cam))
            .add_system_set(
                SystemSet::on_update(AppState::Panel)
                    .with_system(toggle_building)
                    .with_system(handle_harv_blueprint),
            )
            .add_event::<StopBuildingHarvesters>();
    }
}

fn set_up_panel(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    panel_assets: Res<PanelAssetHandlers>,
) {
    let ase_file = asefile::AsepriteFile::read(Cursor::new(include_bytes!(
        "../assets/spritepanel8.aseprite"
    )))
    .expect("valid aseprite");
    let main_panel = bevy_image_from_ase_image(
        ase_file
            .layer_by_name("main")
            .expect("main layer")
            .frame(0)
            .image(),
    );

    let panel_size = main_panel.size() * PIXEL_MULTIPLIER;

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(panel_size),
            ..default()
        },
        transform: Transform {
            translation: PANEL_OFFSET,
            ..default()
        },
        texture: textures.add(main_panel),
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
                    x: PANEL_OFFSET.x,
                    y: PANEL_OFFSET.y,
                    z: 100.0,
                },
                ..default()
            },
            ..default()
        },
        PanelMarker,
    ));

    commands.insert_resource(PanelState {
        building_harvester: false,
    });

    for (i, slot) in panel_assets.harv_slots.iter().enumerate() {
        let empty = &slot[0];
        commands.spawn((
            PanelMarker,
            SlotNumber(i),
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(empty.1),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        z: 1.0,
                        ..PANEL_OFFSET
                    },
                    ..default()
                },
                texture: empty.0.clone(),
                ..default()
            },
        ));
    }
}

fn enable_panel_cam(
    mut panel_cam: Query<&mut Camera, With<PanelMarker>>,
    mut other_cams: Query<&mut Camera, Without<PanelMarker>>,
) {
    panel_cam.for_each_mut(|mut c| c.is_active = true);
    other_cams.for_each_mut(|mut c| c.is_active = false);
}

struct StopBuildingHarvesters;

fn toggle_building(
    mut commands: Commands,
    mut panel_state: ResMut<PanelState>,
    keys: Res<Input<KeyCode>>,
    panel_assets: Res<PanelAssetHandlers>,
    blueprints: Query<Entity, With<HarvesterBlueprint>>,
    mut stopper: EventReader<StopBuildingHarvesters>,
) {
    if keys.just_pressed(KeyCode::B) {
        panel_state.building_harvester = !panel_state.building_harvester;

        if panel_state.building_harvester {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(panel_assets.center_icon[0].1),
                        ..default()
                    },
                    texture: panel_assets.center_icon[0].0.clone(),
                    ..default()
                },
                HarvesterBlueprint,
            ));
        } else {
            blueprints.for_each(|b| commands.entity(b).despawn())
        }
    }

    if panel_state.building_harvester
        && (keys.just_pressed(KeyCode::Escape) || stopper.iter().count() > 0)
    {
        blueprints.for_each(|b| commands.entity(b).despawn());
        panel_state.building_harvester = false;
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_harv_blueprint(
    mut commands: Commands,
    mut harv_blueprint: Query<&mut Transform, With<HarvesterBlueprint>>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    buttons: Res<Input<MouseButton>>,
    terrain_assets: Res<TerrainAssetHandlers>,
    panel_assets: Res<PanelAssetHandlers>,
    mut stopper: EventWriter<StopBuildingHarvesters>,
    mut harvesters: ResMut<TotalHarvesters>,
    mut slot_sprites: Query<(Entity, &mut Handle<Image>, &SlotNumber), With<PanelMarker>>,
    panel_state: Res<PanelState>,
) {
    let Some((camera, camera_transform)) = q_camera.iter().find(|(c,_)|c.is_active) else {return};
    let Some(world_cursor_pos) = get_cursor_pos_in_world_coord(wnds.get_primary().unwrap(), camera_transform, camera) else {return};

    let step = 10.0 * PIXEL_MULTIPLIER;
    let icon_to_panel_sprite_offset = Vec2 { x: 5.5, y: 5.5 } * PIXEL_MULTIPLIER;
    let icon_to_panel_center_offset = Vec2 { x: -8.0, y: -6.0 };

    let cell_on_panel = ((world_cursor_pos - icon_to_panel_sprite_offset) / step).round() * step
        + icon_to_panel_sprite_offset;

    let hovered_cell_coord =
        (cell_on_panel - PANEL_OFFSET.truncate() - icon_to_panel_sprite_offset) / step
            - icon_to_panel_center_offset;

    let clamped_hovered_cell_coord =
        hovered_cell_coord.clamp(Vec2 { x: 1.0, y: 1.0 }, Vec2 { x: 9.0, y: 6.0 });

    let world_coord_on_panel = (clamped_hovered_cell_coord + icon_to_panel_center_offset) * step
        + icon_to_panel_sprite_offset
        + PANEL_OFFSET.truncate();

    harv_blueprint.for_each_mut(|mut t| t.translation = world_coord_on_panel.extend(2.0));
    if buttons.just_pressed(MouseButton::Left) && panel_state.building_harvester {
        let (slot_entity, mut slot_image_handler, slot_number) = {
            let s = slot_sprites
                .iter_mut()
                .find(|(_e, _h, slot_number)| slot_number.0 == harvesters.0);
            match s {
                Some(s) => s,
                None => slot_sprites.iter_mut().last().unwrap(),
            }
        };

        *slot_image_handler = panel_assets.harv_slots[slot_number.0][1].0.clone();

        let center_icon = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(panel_assets.center_icon[0].1),
                    ..default()
                },
                texture: panel_assets.center_icon[0].0.clone(),
                transform: Transform {
                    translation: world_coord_on_panel.extend(1.0),
                    ..default()
                },
                ..default()
            })
            .id();

        // TODO if placement valid
        add_harvester(
            commands,
            terrain_assets,
            (
                clamped_hovered_cell_coord.x as i8,
                clamped_hovered_cell_coord.y as i8,
            ),
            harvesters.0,
            SlotIcon(slot_entity),
            CenterIcon(center_icon),
        );
        harvesters.0 += 1;
        stopper.send(StopBuildingHarvesters);
    }
}
