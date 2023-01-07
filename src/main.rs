use std::io::Cursor;

use bevy::prelude::*;
use image::{DynamicImage, ImageBuffer};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(setup_terrain)
        .add_system(sprite_movement)
        .run();
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                custom_size: Some(Vec2 { x: 300.0, y: 300.0 }),
                ..default()
            },
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::Up,
    ));
}

fn setup_terrain(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    commands.spawn(SpriteBundle {
        texture: textures.add(image_from_aseprite(include_bytes!(
            "../assets/placeholders/terrain.aseprite"
        ))),
        ..default()
    });
}

fn image_from_aseprite(ase_bytes: &[u8]) -> Image {
    let image = asefile::AsepriteFile::read(Cursor::new(ase_bytes))
        .expect("valid aseprite")
        .layers()
        .next()
        .expect("at least one layer")
        .frame(0)
        .image();
    let img_buf = ImageBuffer::from_raw(image.width(), image.height(), image.into_raw())
        .expect("size of containers to match");
    Image::from_dynamic(DynamicImage::ImageRgba8(img_buf), true)
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Up;
        }
    }
}
