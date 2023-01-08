use std::io::Cursor;

use bevy::{ecs::query::ROQueryItem, render::camera::RenderTarget, sprite::collide_aabb::collide};

use crate::{
    buggy::Buggy,
    harvester::{
        add_harvester, CenterIcon, SlotIcon, SlotNumber, StorageHelium, StoredCanisters,
        TotalHarvesters,
    },
    start::EndTimer,
    terrain::{
        CANISTERS_TO_WIN, HELIUM_TO_BUILD_HARVESTER, HELIUM_TO_MAKE_CANISTER, MAX_HELIUM_STORAGE,
    },
    tooltip::TooltipString,
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

pub const CELL_SIZE_PANEL: f32 = 10.0;

#[derive(Component)]
pub struct PanelMarker;

pub struct PanelPlugin;

#[derive(Resource)]
struct PanelState {
    building_harvester: bool,
}

#[derive(Component)]
struct BuggyIcon;

#[derive(Component)]
struct TankLevel;

#[derive(Component)]
struct CanisterButtonSprite;

#[derive(Component)]
struct CanisterButtonText;

#[derive(Component)]
struct CanisterButtonSensor;

#[derive(Component)]
struct StoredCanister;

#[derive(Component)]
struct HarvesterBlueprint;

#[derive(Component)]
pub struct SwitchToTerrainButton;

#[derive(Component)]
pub struct HarvesterButtonText;

#[derive(Component)]
pub struct BuildHarvesterButtonSensor;

#[derive(Component)]
pub struct Ship {
    start: Vec3,
    finish: Vec3,
}

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(AppState::Start).with_system(set_up_panel))
            .add_system_set(SystemSet::on_enter(AppState::Panel).with_system(enable_panel_cam))
            .add_system_set(
                SystemSet::on_update(AppState::Panel)
                    .with_system(toggle_building)
                    .with_system(move_buggy_on_map)
                    .with_system(handle_harv_blueprint.after(mouse_clicks_panel))
                    .with_system(mouse_clicks_panel)
                    .with_system(canister_builder)
                    .with_system(update_ship)
                    .with_system(update_tank_level),
            )
            .add_event::<StopBuildingHarvesters>()
            .add_event::<EnterBuildingHarvestersMode>()
            .add_event::<MakeCanister>();
    }
}

pub fn tank_center() -> Vec3 {
    Vec3 {
        z: 1.0,
        ..PANEL_OFFSET
    } + Vec3 {
        x: 122.0 - 80.0,
        y: 60.0 - 103.0,
        z: 0.0,
    } * PIXEL_MULTIPLIER
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

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.exit.1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x,
                    y: PANEL_OFFSET.y,
                    z: 3.0,
                },
                ..default()
            },
            texture: panel_assets.exit.0.clone(),
            ..default()
        })
        .insert(PanelMarker);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(17.0 * PIXEL_MULTIPLIER, 11.0 * PIXEL_MULTIPLIER)),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x - WIDTH / 2.0 + 11.0 * PIXEL_MULTIPLIER,
                    y: PANEL_OFFSET.y + HEIGHT / 2.0 - 8.0 * PIXEL_MULTIPLIER,
                    z: -100.0,
                },
                ..default()
            },
            ..default()
        })
        .insert(SwitchToTerrainButton)
        .insert(TooltipString("Back to vehicle".to_string()))
        .insert(PanelMarker);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.harvester_button[0].1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    z: 1.0,
                    ..PANEL_OFFSET
                },
                ..default()
            },
            texture: panel_assets.harvester_button[0].0.clone(),
            ..default()
        })
        .insert(PanelMarker);

    commands.spawn((
        BuildHarvesterButtonSensor,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(42.0 * PIXEL_MULTIPLIER, 9.0 * PIXEL_MULTIPLIER)),
                color: Color::rgba(0.0, 1.0, 1.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x - WIDTH / 2.0 + 135.0 * PIXEL_MULTIPLIER,
                    y: PANEL_OFFSET.y + HEIGHT / 2.0 - 77.5 * PIXEL_MULTIPLIER,
                    z: 3.0,
                },
                ..default()
            },
            ..default()
        },
        PanelMarker,
    ));

    commands
        .spawn((
            HarvesterButtonText,
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(panel_assets.harvester_button[1].1),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        z: 2.0,
                        ..PANEL_OFFSET
                    },
                    ..default()
                },
                texture: panel_assets.harvester_button[1].0.clone(),
                ..default()
            },
        ))
        .insert(PanelMarker);

    commands.spawn((
        BuggyIcon,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.buggy_icon.1),
                ..default()
            },
            texture: panel_assets.buggy_icon.0.clone(),
            transform: Transform {
                translation: Vec3 {
                    z: 2.0,
                    ..PANEL_OFFSET
                },
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn((
        TankLevel,
        SpriteBundle {
            sprite: Sprite {
                color: Color::SEA_GREEN,
                custom_size: Some(Vec2 {
                    x: 2.0 * PIXEL_MULTIPLIER,
                    y: 20.0 * PIXEL_MULTIPLIER,
                }),
                ..default()
            },
            transform: Transform {
                translation: tank_center(),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        CanisterButtonSprite,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.tank_button[0].1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    z: 1.0,
                    ..PANEL_OFFSET
                },
                ..default()
            },
            texture: panel_assets.tank_button[0].0.clone(),
            ..default()
        },
    ));

    commands.spawn((
        CanisterButtonText,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.tank_button[1].1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    z: 2.0,
                    ..PANEL_OFFSET
                },
                ..default()
            },
            texture: panel_assets.tank_button[1].0.clone(),
            ..default()
        },
    ));

    commands.spawn((
        CanisterButtonSensor,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(27.0 * PIXEL_MULTIPLIER, 9.0 * PIXEL_MULTIPLIER)),
                color: Color::rgba(0.0, 1.0, 1.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x - WIDTH / 2.0 + 142.5 * PIXEL_MULTIPLIER,
                    y: PANEL_OFFSET.y + HEIGHT / 2.0 - 89.5 * PIXEL_MULTIPLIER,
                    z: 3.0,
                },
                ..default()
            },
            ..default()
        },
    ));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.ship.1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x - WIDTH / 2.0 + 40.0,
                    y: PANEL_OFFSET.y + HEIGHT / 2.0 - 120.0,
                    z: 3.0,
                },
                ..default()
            },
            texture: panel_assets.ship.0.clone(),
            ..default()
        })
        .insert(Ship {
            start: Vec3 {
                x: PANEL_OFFSET.x - WIDTH / 2.0 + 40.0,
                y: PANEL_OFFSET.y + HEIGHT / 2.0 - 120.0,
                z: 3.0,
            },
            finish: Vec3 {
                x: PANEL_OFFSET.x + 80.0,
                y: PANEL_OFFSET.y + 200.0,
                z: 3.0,
            },
        })
        .insert(TooltipString("ship".to_string()))
        .insert(PanelMarker);
}

fn enable_panel_cam(
    mut panel_cam: Query<&mut Camera, With<PanelMarker>>,
    mut other_cams: Query<&mut Camera, Without<PanelMarker>>,
) {
    panel_cam.for_each_mut(|mut c| c.is_active = true);
    other_cams.for_each_mut(|mut c| c.is_active = false);
}

struct StopBuildingHarvesters;
struct EnterBuildingHarvestersMode;
struct MakeCanister;

fn toggle_building(
    mut commands: Commands,
    mut panel_state: ResMut<PanelState>,
    keys: Res<Input<KeyCode>>,
    panel_assets: Res<PanelAssetHandlers>,
    blueprints: Query<Entity, With<HarvesterBlueprint>>,
    mut stopper: EventReader<StopBuildingHarvesters>,
    mut starter: EventReader<EnterBuildingHarvestersMode>,
) {
    if keys.just_pressed(KeyCode::B) || starter.iter().count() > 0 {
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
    mut helium: ResMut<StorageHelium>,
) {
    let Some((camera, camera_transform)) = q_camera.iter().find(|(c,_)|c.is_active) else {return};
    let Some(world_cursor_pos) = get_cursor_pos_in_world_coord(wnds.get_primary().unwrap(), camera_transform, camera) else {return};

    let (cell_coord, world_coord_on_panel) =
        panel_coord_to_cell_and_snapped_panel_world_coord(world_cursor_pos);

    harv_blueprint.for_each_mut(|mut t| t.translation = world_coord_on_panel.extend(2.0));
    if buttons.just_pressed(MouseButton::Left) && panel_state.building_harvester {
        // TODO if placement valid
        if helium.0 < HELIUM_TO_BUILD_HARVESTER {
            stopper.send(StopBuildingHarvesters);
            return;
        }
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

        helium.0 -= HELIUM_TO_BUILD_HARVESTER;
        add_harvester(
            commands,
            terrain_assets,
            cell_coord,
            harvesters.0,
            SlotIcon(slot_entity),
            CenterIcon(center_icon),
        );
        harvesters.0 += 1;
        stopper.send(StopBuildingHarvesters);
    }
}

#[allow(clippy::too_many_arguments)]
fn mouse_clicks_panel(
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut buttons: ResMut<Input<MouseButton>>,
    mut app_state: ResMut<State<AppState>>,
    terrain_button: Query<(&Transform, &Sprite), With<SwitchToTerrainButton>>,
    harvester_button: Query<(&Transform, &Sprite), With<BuildHarvesterButtonSensor>>,
    canister_button: Query<(&Transform, &Sprite), With<CanisterButtonSensor>>,
    helium: Res<StorageHelium>,
    mut building_starter: EventWriter<EnterBuildingHarvestersMode>,
    mut canister_builder: EventWriter<MakeCanister>,
) {
    let Some((camera, camera_transform)) = q_camera.iter().find(|(c,_)|c.is_active) else {return};

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    if buttons.just_pressed(MouseButton::Left) {
        let cursor_collider = Vec2 { x: 1.0, y: 1.0 };
        if let Some(screen_pos) = wnd.cursor_position() {
            let window_size = Vec2::new(wnd.width(), wnd.height());
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            let clicks_sprite = |q: ROQueryItem<(&Transform, &Sprite)>| {
                collide(
                    q.0.translation,
                    Vec2 {
                        x: q.1.custom_size.unwrap().x,
                        y: q.1.custom_size.unwrap().y,
                    },
                    world_pos,
                    cursor_collider,
                )
                .is_some()
            };

            if clicks_sprite(terrain_button.single()) {
                app_state.set(AppState::Terrain).unwrap();
                buttons.clear();
                return;
            }

            if clicks_sprite(harvester_button.single()) && helium.0 >= HELIUM_TO_BUILD_HARVESTER {
                building_starter.send(EnterBuildingHarvestersMode);
                buttons.clear();
            }

            if clicks_sprite(canister_button.single()) && helium.0 >= HELIUM_TO_MAKE_CANISTER {
                canister_builder.send(MakeCanister);
            }
        }
    }
}

fn move_buggy_on_map(
    buggy: Query<&Transform, With<Buggy>>,
    mut buggy_icon: Query<&mut Transform, (With<BuggyIcon>, Without<Buggy>)>,
) {
    let Ok(pos) = buggy.get_single() else {return};
    let Ok(mut buggy_icon_pos) = buggy_icon.get_single_mut() else {return};

    // center of the grid - center of the sprite
    let center_offset = Vec2 {
        x: (80.0 - 55.0) * PIXEL_MULTIPLIER,
        y: (79.0 - 60.0) * PIXEL_MULTIPLIER,
    };

    buggy_icon_pos.translation = pos.translation / CELL_SIZE_TERRAIN * CELL_SIZE_PANEL
        + PANEL_OFFSET
        - center_offset.extend(0.0);
}

fn panel_coord_to_cell_and_snapped_panel_world_coord(world_coord: Vec2) -> ((i8, i8), Vec2) {
    let step = CELL_SIZE_PANEL * PIXEL_MULTIPLIER;
    let icon_to_panel_sprite_offset = Vec2 { x: 5.5, y: 5.5 } * PIXEL_MULTIPLIER;
    let icon_to_panel_center_offset = Vec2 { x: -8.0, y: -6.0 };

    let cell_on_panel = ((world_coord - icon_to_panel_sprite_offset) / step).round() * step
        + icon_to_panel_sprite_offset;

    let cell_coord = (cell_on_panel - PANEL_OFFSET.truncate() - icon_to_panel_sprite_offset) / step
        - icon_to_panel_center_offset;

    let clamped_cell_coord = cell_coord.clamp(Vec2 { x: 1.0, y: 1.0 }, Vec2 { x: 9.0, y: 6.0 });

    let world_coord_on_panel = (clamped_cell_coord + icon_to_panel_center_offset) * step
        + icon_to_panel_sprite_offset
        + PANEL_OFFSET.truncate();
    (
        (clamped_cell_coord.x as i8, clamped_cell_coord.y as i8),
        world_coord_on_panel,
    )
}

fn update_tank_level(
    mut tank: Query<(&mut Sprite, &mut Transform), With<TankLevel>>,
    helium: Res<StorageHelium>,
    mut new_harv_button: Query<&mut Handle<Image>, With<HarvesterButtonText>>,
    mut canister_button: Query<
        &mut Handle<Image>,
        (With<CanisterButtonText>, Without<HarvesterButtonText>),
    >,
    panel_assets: Res<PanelAssetHandlers>,
) {
    let max_tank_height_px = 20.0;
    let progress = helium.0 as f32 / MAX_HELIUM_STORAGE as f32;
    let height = (max_tank_height_px * progress).round();

    let (mut sprite, mut transform) = tank.single_mut();

    transform.translation.y = tank_center().y - (10.0 - height / 2.0) * PIXEL_MULTIPLIER;
    sprite.custom_size.as_mut().unwrap().y = height * PIXEL_MULTIPLIER;
    let button_text_img_idx = match helium.0 >= HELIUM_TO_BUILD_HARVESTER {
        true => 2,
        false => 1,
    };
    *new_harv_button.single_mut() = panel_assets.harvester_button[button_text_img_idx].0.clone();

    *canister_button.single_mut() =
        panel_assets.tank_button[match helium.0 >= HELIUM_TO_MAKE_CANISTER {
            true => 2,
            false => 1,
        }]
        .0
        .clone();
}

fn canister_builder(
    mut commands: Commands,
    mut canister_event: EventReader<MakeCanister>,
    mut helium: ResMut<StorageHelium>,
    mut stored_canisters: ResMut<StoredCanisters>,
    panel_assets: Res<PanelAssetHandlers>,
    mut state: ResMut<State<AppState>>,
) {
    for _ in canister_event.iter() {
        if helium.0 < HELIUM_TO_MAKE_CANISTER {
            return;
        }
        helium.0 -= HELIUM_TO_MAKE_CANISTER;
        let (can_img, size) =
            &panel_assets.tanks[stored_canisters.0.min(panel_assets.tanks.len() - 1)];
        commands.spawn((
            StoredCanister,
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(*size),
                    ..default()
                },
                texture: can_img.clone(),
                transform: Transform {
                    translation: Vec3 {
                        z: 1.0,
                        ..PANEL_OFFSET
                    },
                    ..default()
                },
                ..default()
            },
        ));
        stored_canisters.0 += 1;
        if stored_canisters.0 == CANISTERS_TO_WIN {
            state.set(AppState::Finish).unwrap();
            return;
        }
    }
}

fn update_ship(
    mut ship: Query<(&mut Transform, &mut TooltipString, &Ship), With<Ship>>,
    timer: Query<&EndTimer>,
) {
    let (mut ship_transform, mut string, ship) = ship.single_mut();
    let timer = timer.single();
    ship_transform.translation.x =
        ship.start.x + ((ship.finish.x - ship.start.x) * timer.timer.percent());
    ship_transform.translation.y =
        ship.start.y + ((ship.finish.y - ship.start.y) * timer.timer.percent());
    string.0 = format!(
        "{} seconds left before arriving",
        timer.timer.remaining_secs() as i32
    );
}

fn spawn_ship(mut commands: Commands, panel_assets: Res<PanelAssetHandlers>) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(panel_assets.ship.1),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    x: PANEL_OFFSET.x - WIDTH / 2.0 + 40.0,
                    y: PANEL_OFFSET.y + HEIGHT / 2.0 - 120.0,
                    z: 3.0,
                },
                ..default()
            },
            texture: panel_assets.ship.0.clone(),
            ..default()
        })
        .insert(Ship {
            start: Vec3 {
                x: PANEL_OFFSET.x - WIDTH / 2.0 + 40.0,
                y: PANEL_OFFSET.y + HEIGHT / 2.0 - 120.0,
                z: 3.0,
            },
            finish: Vec3 {
                x: PANEL_OFFSET.x + 80.0,
                y: PANEL_OFFSET.y + 200.0,
                z: 3.0,
            },
        })
        .insert(TooltipString("ship".to_string()))
        .insert(PanelMarker);
}
