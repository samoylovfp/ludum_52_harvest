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
        app.add_system_set(SystemSet::on_enter(AppState::Terrain).with_system(setup_terrain));
    }
}

fn setup_terrain(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let mut image = image_from_aseprite(include_bytes!("../assets/placeholders/terrain.aseprite"));

    // Do not interpolate the image
    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: bevy::render::render_resource::FilterMode::Nearest,
        min_filter: bevy::render::render_resource::FilterMode::Nearest,
        ..default()
    });
    
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
