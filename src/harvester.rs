use crate::{terrain::TerrainMarker, util::image_from_aseprite};
use bevy::{
    prelude::*,
    render::{render_resource::SamplerDescriptor, texture::ImageSampler},
};

use super::*;

pub fn add_harvester(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    cell: (i8, i8),
    slot: usize,
) {
    let mut image = image_from_aseprite(include_bytes!("../assets/centerv1.aseprite"));

    // Do not interpolate the image
    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: bevy::render::render_resource::FilterMode::Nearest,
        min_filter: bevy::render::render_resource::FilterMode::Nearest,
        ..default()
    });

    let size = image.size() * PIXEL_MULTIPLIER;
    let center_coords = (
        cell.0 as f32 * PIXEL_MULTIPLIER,
        cell.1 as f32 * PIXEL_MULTIPLIER,
    );
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
            texture: textures.add(image.clone()),

            ..default()
        })
        .insert(Harvester)
        .insert(Cell(cell))
        .insert(Moves(true))
        .insert(Direction::Right)
        .insert(TerrainMarker)
        .id();

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: textures.add(image),
            transform: Transform {
                translation: Vec3::new(center_coords.0, center_coords.1, 1.0),
                ..Default::default()
            },
            ..default()
        })
        .insert(Center)
        .insert(HarvesterId(harvester_id))
        .insert(HarvesterState::Work)
        .insert(HarvestTime(0))
        .insert(Helium(0))
        .insert(SlotNumber(slot))
        .insert(TerrainMarker);
}

pub fn move_harvesters(
    mut harvesters: Query<(&Cell, &mut Transform, &mut Direction), With<TerrainMarker>>,
) {
    for (cell, mut transform, mut direction) in harvesters.iter_mut() {
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
            (o1, o2) if o1 == offset && o2 == offset => {*direction = Direction::Down; transform.rotate_z(angle)},
            (o1, o2) if o1 == offset && o2 == -offset => {*direction = Direction::Left; transform.rotate_z(angle)},
            (o1, o2) if o1 == -offset && o2 == -offset => {*direction = Direction::Up; transform.rotate_z(angle)},
            (o1, o2) if o1 == -offset && o2 == offset => {*direction = Direction::Right; transform.rotate_z(angle)},
            _ => (),
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
pub struct Helium(usize);

#[derive(Component)]
pub struct HarvestTime(usize);

#[derive(Component)]
pub enum HarvesterState {
    Work,
    Full,
    Broken,
}
