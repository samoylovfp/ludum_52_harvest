use crate::{
    harvester::{add_harvester, move_harvesters},
    util::image_from_aseprite,
    AppState, PIXEL_MULTIPLIER,
};
use bevy::{
    prelude::*,
    render::{render_resource::SamplerDescriptor, texture::ImageSampler},
};

#[derive(Component)]
pub struct TerrainMarker;

pub struct Terrain;

#[derive(Component)]
pub struct Buggy {
    velocity: f32,
}

impl Plugin for Terrain {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Terrain)
                .with_system(setup_terrain)
                .with_system(setup_buggy),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Terrain)
                .with_system(move_harvesters)
                .with_system(buggy_movement_and_control),
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

    add_harvester(commands, textures, (0, 0), 0);
}

fn setup_buggy(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let buggy_image = image_from_aseprite(include_bytes!("../assets/buggyv3.aseprite"));
    let size = buggy_image.size() * PIXEL_MULTIPLIER;
    commands
        .spawn(SpriteBundle {
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
        })
        .insert(Buggy {
            velocity: default(),
        });
}

fn buggy_movement_and_control(
    mut buggy: Query<(&mut Buggy, &mut Transform), Without<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    keys: Res<Input<KeyCode>>,
) {
    let mut position = None;

    let max_fwd_speed = 20.0;
    let max_bck_speed = -2.0;
    let speedup = 0.5;
    let friction = 0.05;
    let stop_thresh = 0.01;
    let max_turn_rate = 0.03;

    if let Ok((mut buggy, mut buggy_pos)) = buggy.get_single_mut() {
        let turn_rate = (buggy.velocity.abs() / 50.0).min(max_turn_rate);
        if keys.pressed(KeyCode::W) {
            buggy.velocity += speedup;
        }
        if keys.pressed(KeyCode::S) {
            buggy.velocity -= speedup;
        }
        if keys.pressed(KeyCode::A) {
            buggy_pos.rotation =
                (buggy_pos.rotation * Quat::from_rotation_z(turn_rate)).normalize();
        }
        if keys.pressed(KeyCode::D) {
            buggy_pos.rotation =
                (buggy_pos.rotation * Quat::from_rotation_z(-turn_rate)).normalize();
        }
        buggy.velocity =
            (buggy.velocity - friction * buggy.velocity).clamp(max_bck_speed, max_fwd_speed);

        if buggy.velocity.abs() > stop_thresh {
            let mut buggy_heading = buggy_pos.rotation
                * Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                };

            // We don't want our car to move about Z axis
            buggy_heading.z = 0.0;

            buggy_pos.translation += buggy_heading * buggy.velocity;
            position = Some(buggy_pos.translation);
        }
    }
    if let Some(pos) = position {
        camera.get_single_mut().unwrap().translation = pos;
    }
}
