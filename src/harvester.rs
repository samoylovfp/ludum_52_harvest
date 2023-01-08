use crate::{terrain::TerrainMarker, tooltip::TooltipString, util::image_from_aseprite};
use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};
use once_cell::sync::OnceCell;
use rand::{thread_rng, Rng};

use super::*;

pub const HARVEST_SPEED: usize = 100;
pub const MAX_HELIUM: usize = 10;

pub fn add_harvester(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    cell: (i8, i8),
    slot: usize,
) {
    static HARV_IMAGE_CELL: OnceCell<Image> = OnceCell::new();
    static HARV_TEXTURE_HANDLE_CELL: OnceCell<Handle<Image>> = OnceCell::new();

    let harv_image = HARV_IMAGE_CELL.get_or_init(|| {
        image_from_aseprite(include_bytes!("../assets/spritecenter1.aseprite"), "base")
    });
    let harv_texture_handle =
        HARV_TEXTURE_HANDLE_CELL.get_or_init(|| textures.add(harv_image.clone()));
    let size = harv_image.size() * PIXEL_MULTIPLIER;

    let center_coords = (
        cell.0 as f32 * PIXEL_MULTIPLIER,
        cell.1 as f32 * PIXEL_MULTIPLIER,
    );

    let mut rng = thread_rng();

    let harvester_id = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    center_coords.0 - (CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER),
                    center_coords.1 + (CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER),
                    1.0,
                ),
                ..Default::default()
            },
            texture: harv_texture_handle.clone(),

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

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: harv_texture_handle.clone(),
            transform: Transform {
                translation: Vec3::new(center_coords.0, center_coords.1, 1.0),
                ..Default::default()
            },
            ..default()
        })
        .insert(Center)
        .insert(BreakTime(rng.gen_range(100..1000)))
        .insert(HarvesterId(harvester_id))
        .insert(HarvesterState::Work)
        .insert(HarvestTime(0))
        .insert(Helium(0))
        .insert(SlotNumber(slot))
        .insert(TooltipString("Collecting...".to_string()))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(12.5 * PIXEL_MULTIPLIER, 12.5 * PIXEL_MULTIPLIER),
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
            cell.0 .0 as f32 * PIXEL_MULTIPLIER,
            cell.0 .1 as f32 * PIXEL_MULTIPLIER,
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

pub fn update_center(
    mut centers: Query<
        (
            &HarvesterId,
            &SlotNumber,
            &mut HarvesterState,
            &mut HarvestTime,
            &mut Helium,
            &mut TooltipString,
            &mut BreakTime,
        ),
        (With<Center>, Without<Harvester>),
    >,
    mut harvesters: Query<(&mut Moves, &mut TooltipString), (With<Harvester>, Without<Center>)>,
) {
    for (harvester_id, slot, mut state, mut time, mut helium, mut string, mut breaktime) in
        centers.iter_mut()
    {
        let (mut harvester, mut harv_string) = harvesters.get_mut(harvester_id.0).unwrap();
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
            }
            HarvesterState::Full => {
                string.0 = format!(
                    "Harvester {}\nStatus: Full\nClick to collect helium",
                    slot.0
                );
                harvester.0 = false;
                harv_string.0 = "Waiting...".to_string();
            }
            HarvesterState::Broken => {
                string.0 = format!("Harvester {}\nStatus: Broken\nClick to repair", slot.0);
                harvester.0 = false;
                harv_string.0 = "Waiting...".to_string();
            }
        }
    }
}

#[derive(Component)]
pub struct Harvester;

#[derive(Component)]
pub struct Cell((i8, i8));

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
pub struct SlotNumber(usize);

#[derive(Component)]
pub struct HarvesterId(Entity);

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
