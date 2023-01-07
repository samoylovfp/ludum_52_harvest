use crate::{util::image_from_aseprite, AppState, PIXEL_MULTIPLIER};
use bevy::{
    prelude::*,
    render::{render_resource::SamplerDescriptor, texture::ImageSampler},
};

#[derive(Component)]
pub struct TerrainMarker;

pub struct Terrain;

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Terrain)
                .with_system(setup_terrain)
                .with_system(setup_buggy),
        );
    }
}

fn setup_terrain(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let image = image_from_aseprite(include_bytes!("../assets/placeholders/terrain.aseprite"));
    let size = image.size() * PIXEL_MULTIPLIER;
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: textures.add(image),
            ..default()
        })
        .insert(TerrainMarker);
}

fn setup_buggy(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let buggy_image = image_from_aseprite(include_bytes!("../assets/buggyv3.aseprite"));
    let size = buggy_image.size() * PIXEL_MULTIPLIER;
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(size),
            ..default()
        },
        transform: Transform {
            translation: Vec3 {
                z: 1.0,
                ..default()
            },
            ..default()
        },
        texture: textures.add(buggy_image),
        ..default()
    });
}
