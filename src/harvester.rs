use std::f32::consts::FRAC_PI_2;

use crate::{
    terrain::{TerrainMarker, TERRAIN_SIZE},
    tooltip::TooltipString,
    util::{PanelAssetHandlers, TerrainAssetHandlers},
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};
use rand::{thread_rng, Rng};

use super::*;

pub const HARVEST_SPEED: usize = 30;
pub const MAX_HELIUM: usize = 30;
pub const BREAKTIME: (i32, i32) = (300, 2000);

pub fn add_harvester(
    mut commands: Commands,
    terrain_assets: Res<TerrainAssetHandlers>,
    cell: (i8, i8),
    slot: usize,
    slot_icon: SlotIcon,
    center_icon: CenterIcon,
) {
    let center_coords = (
        cell.0 as f32 * CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER - TERRAIN_SIZE.0 / 2.0
            + CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
        cell.1 as f32 * CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER - TERRAIN_SIZE.1 / 2.0
            + CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
    );

    let mut rng = thread_rng();

    let harvester_id = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(terrain_assets.harvester.1),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    center_coords.0 - (CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER),
                    center_coords.1 + (CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER),
                    1.0,
                ),
                rotation: Quat::from_rotation_z(-FRAC_PI_2),
                ..Default::default()
            },
            texture: terrain_assets.harvester.0.clone(),

            ..default()
        })
        .insert(Harvester)
        .insert(Cell(cell))
        .insert(Moves(true))
        .insert(Direction::Right)
        .insert(TerrainMarker)
        .insert((
            RigidBody::KinematicPositionBased,
            Collider::cuboid(10.0 * PIXEL_MULTIPLIER, 10.0 * PIXEL_MULTIPLIER),
        ))
        .insert(TooltipString("Working...".to_string()))
        .id();

    let lamp_id = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(terrain_assets.center.1),
                ..default()
            },
            texture: terrain_assets.center_terrain_lamps[2].0.clone(),
            transform: Transform {
                translation: Vec3::new(center_coords.0, center_coords.1, 1.5),
                ..Default::default()
            },
            ..default()
        })
        .insert(Lamp)
        .insert(TerrainMarker)
        .id();

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(terrain_assets.center.1),
                ..default()
            },
            texture: terrain_assets.center.0.clone(),
            transform: Transform {
                translation: Vec3::new(center_coords.0, center_coords.1, 1.0),
                ..Default::default()
            },
            ..default()
        })
        .insert(Center)
        .insert(BreakTime(rng.gen_range(BREAKTIME.0..BREAKTIME.1)))
        .insert(HarvesterId(harvester_id))
        .insert(slot_icon)
        .insert(center_icon)
        .insert(LampId(lamp_id))
        .insert(HarvesterState::Work)
        .insert(HarvestTime(0))
        .insert(Helium(0))
        .insert(SlotNumber(slot))
        .insert(TooltipString("Collecting...".to_string()))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(11.0 * PIXEL_MULTIPLIER, 11.0 * PIXEL_MULTIPLIER),
        ))
        .insert(TerrainMarker);
}

pub fn move_harvesters(
    mut harvesters: Query<(&Cell, &mut Transform, &mut Direction, &Moves), With<TerrainMarker>>,
) {
    for (cell, mut transform, mut direction, moves) in harvesters.iter_mut() {
        if !moves.0 {
            continue;
        }
        let current_cell = (
            cell.0 .0 as f32 * CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER - TERRAIN_SIZE.0 / 2.0
                + CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
            cell.0 .1 as f32 * CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER - TERRAIN_SIZE.1 / 2.0
                + CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
        );
        match *direction {
            Direction::Up => transform.translation.y += 1.0,
            Direction::Right => transform.translation.x += 1.0,
            Direction::Down => transform.translation.y -= 1.0,
            Direction::Left => transform.translation.x -= 1.0,
        };
        let offset = (CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER) as i32;
        let angle = -90.0_f32.to_radians();
        match (
            (transform.translation.x - current_cell.0) as i32,
            (transform.translation.y - current_cell.1) as i32,
        ) {
            (o1, o2) if o1 == offset && o2 == offset => {
                *direction = Direction::Down;
                transform.rotate_z(angle)
            }
            (o1, o2) if o1 == offset && o2 == -offset => {
                *direction = Direction::Left;
                transform.rotate_z(angle)
            }
            (o1, o2) if o1 == -offset && o2 == -offset => {
                *direction = Direction::Up;
                transform.rotate_z(angle)
            }
            (o1, o2) if o1 == -offset && o2 == offset => {
                *direction = Direction::Right;
                transform.rotate_z(angle)
            }
            _ => (),
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_center(
    mut centers: Query<
        (
            &HarvesterId,
            &LampId,
            &SlotNumber,
            &SlotIcon,
            &CenterIcon,
            &mut HarvesterState,
            &mut HarvestTime,
            &mut Helium,
            &mut TooltipString,
            &mut BreakTime,
        ),
        (With<Center>, Without<Harvester>),
    >,
    mut harvesters: Query<(&mut Moves, &mut TooltipString), (With<Harvester>, Without<Center>)>,
    terrain_assets: Res<TerrainAssetHandlers>,
    panel_assets: Res<PanelAssetHandlers>,
    mut imgs: Query<&mut Handle<Image>>,
) {
    for (
        harvester_id,
        lamp_id,
        slot,
        slot_icon,
        center_icon,
        mut state,
        mut time,
        mut helium,
        mut string,
        mut breaktime,
    ) in centers.iter_mut()
    {
        let (mut harvester, mut harv_string) = harvesters.get_mut(harvester_id.0).unwrap();
        let mut lamp = imgs.get_mut(lamp_id.0).unwrap();
        if helium.0 == MAX_HELIUM {
            *state = HarvesterState::Full;
        }
        if breaktime.0 <= 0 {
            *state = HarvesterState::Broken;
        }
        match *state {
            HarvesterState::Work => {
                time.0 += 1;
                if time.0 >= HARVEST_SPEED {
                    helium.0 += 1;
                    time.0 = 0;
                }
                breaktime.0 -= 1;
                string.0 = format!(
                    "Harvester {}\nStatus: Working\nHelium amount: {}/{}",
                    slot.0, helium.0, MAX_HELIUM
                );
                harvester.0 = true;
                harv_string.0 = "Collecting...".to_string();
                *lamp = terrain_assets.center_terrain_lamps[2].0.clone();
            }
            HarvesterState::Full => {
                string.0 = format!(
                    "Harvester {}\nStatus: Full\nClick to collect helium",
                    slot.0
                );
                harvester.0 = false;
                harv_string.0 = "Waiting...".to_string();
                *lamp = terrain_assets.center_terrain_lamps[1].0.clone();
            }
            HarvesterState::Broken => {
                string.0 = format!("Harvester {}\nStatus: Broken\nClick to repair", slot.0);
                harvester.0 = false;
                harv_string.0 = "Waiting...".to_string();
                *lamp = terrain_assets.center_terrain_lamps[0].0.clone();
            }
        }
        let mut slot_img = imgs.get_mut(slot_icon.0).unwrap();

        let new_slot_img_idx = match *state {
            HarvesterState::Work => 1,
            HarvesterState::Full => 2,
            HarvesterState::Broken => 3,
        };
        *slot_img = panel_assets.harv_slots[slot.0.min(panel_assets.harv_slots.len() - 1)]
            [new_slot_img_idx]
            .0
            .clone();

        let new_center_img_idx = match *state {
            HarvesterState::Work => 0,
            HarvesterState::Full => 1,
            HarvesterState::Broken => 2,
        };

        *imgs.get_mut(center_icon.0).unwrap() =
            panel_assets.center_icon[new_center_img_idx].0.clone();
    }
}

#[derive(Component)]
pub struct Harvester;

#[derive(Component)]
pub struct Cell(pub (i8, i8));

#[derive(Component)]
pub struct Moves(bool);

#[derive(Component)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Component)]
pub struct Center;

#[derive(Component)]
pub struct SlotNumber(pub usize);

#[derive(Component)]
pub struct HarvesterId(Entity);

#[derive(Component)]
pub struct SlotIcon(pub Entity);

#[derive(Component)]
pub struct CenterIcon(pub Entity);

#[derive(Component)]
pub struct LampId(Entity);

#[derive(Component)]
pub struct Helium(pub usize);

#[derive(Component)]
pub struct BreakTime(pub i32);

#[derive(Component)]
pub struct HarvestTime(usize);

#[derive(Component)]
pub enum HarvesterState {
    Work,
    Full,
    Broken,
}

#[derive(Resource)]
pub struct TotalHarvesters(pub usize);

#[derive(Component)]
pub struct Lamp;

#[derive(Resource)]
pub struct StorageHelium(pub usize);

#[derive(Resource)]
pub struct StoredCanisters(pub usize);
