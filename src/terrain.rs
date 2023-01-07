use crate::{
    harvester::{add_harvester, move_harvesters},
    util::image_from_aseprite,
    AppState, PIXEL_MULTIPLIER,
};
use bevy::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::prelude::RigidBodyDamping};

#[derive(Component)]
pub struct TerrainMarker;

pub struct Terrain;

#[derive(Component)]
pub struct Buggy {}

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
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(12.0))
        .add_plugin(RapierDebugRenderPlugin::default());
    }
}

fn setup_terrain(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut phys: ResMut<RapierConfiguration>,
) {
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
    phys.gravity = Vec2 { x: 0.0, y: 0.0 };
}

fn setup_buggy(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let buggy_image = image_from_aseprite(include_bytes!("../assets/spritebuggy3.aseprite"));
    let size = buggy_image.size() * PIXEL_MULTIPLIER;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform {
                translation: Vec3 {
                    z: 1.0,
                    y: 26.0 * PIXEL_MULTIPLIER,
                    ..default()
                },
                ..default()
            },
            texture: textures.add(buggy_image),
            ..default()
        },
        Buggy {},
        RigidBody::Dynamic,
        Damping {
            angular_damping: 0.9,
            linear_damping: 0.1,
        },
        Collider::cuboid(6.0 * PIXEL_MULTIPLIER, 10.0 * PIXEL_MULTIPLIER),
        ColliderMassProperties::Density(2.0),
        Velocity::default(),
        ExternalForce::default(),
        TerrainMarker,
    ));
}

#[allow(clippy::type_complexity)]
fn buggy_movement_and_control(
    mut buggy: Query<(&Velocity, &mut ExternalForce, &Transform), (With<Buggy>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    keys: Res<Input<KeyCode>>,
) {
    let mut position = None;

    let friction = 400.0;
    let max_turn_force = 3000.0;
    let horse_power_fwd = 10_000.0;
    let horse_power_back = 2_000.0;

    if let Ok((vel, mut force, pos)) = buggy.get_single_mut() {
        let buggy_side = pos.rotation
            * Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };
        let buggy_heading = pos.rotation
        * Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let turn_force = (vel.linvel.length() * 10_000.0).min(max_turn_force);

        let mut torque = 0.0;
        let mut acceleration = 0.0;
        force.force = Vec2::default();

        if keys.pressed(KeyCode::W) {
            acceleration = horse_power_fwd;
        }
        if keys.pressed(KeyCode::S) {
            acceleration = -horse_power_back;
        }
        if keys.pressed(KeyCode::A) {
            torque = turn_force;
        }
        if keys.pressed(KeyCode::D) {
            torque = -turn_force;
        }
        force.torque = torque;
        force.force += buggy_heading.truncate() * acceleration;

        let lateral_force = vel.linvel.project_onto(buggy_side.truncate());
        let mut lateral_friction = Vec2::default();

        if lateral_force.length() > 0.01 {
            lateral_friction = -lateral_force * friction;
        }

        force.force += lateral_friction;
        position = Some(pos.translation);

    }
    if let Some(pos) = position {
        camera.get_single_mut().unwrap().translation = pos;
        camera.get_single_mut().unwrap().translation.z = 100.0;
    }
}
