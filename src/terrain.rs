use crate::{
    harvester::{add_harvester, move_harvesters},
    util::image_from_aseprite,
    AppState, PIXEL_MULTIPLIER,
};
use bevy::{asset::HandleId, prelude::*};
use bevy_rapier2d::{prelude::*, rapier::prelude::RigidBodyDamping};
use once_cell::sync::OnceCell;

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
    static TERRAIN_IMAGE_CELL: OnceCell<Image> = OnceCell::new();
    static TERRAIN_TEXTURE_HANDLE_CELL: OnceCell<Handle<Image>> = OnceCell::new();

    let terrain_image = TERRAIN_IMAGE_CELL.get_or_init(|| {
        image_from_aseprite(include_bytes!("../assets/placeholders/terrain.aseprite"))
    });
    let terrain_texture_handle =
        TERRAIN_TEXTURE_HANDLE_CELL.get_or_init(|| textures.add(terrain_image.clone()));
    let size = terrain_image.size() * PIXEL_MULTIPLIER;

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: terrain_texture_handle.clone(),
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
            angular_damping: 0.96,
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
    let horse_power_fwd = 15_000.0;
    let horse_power_back = 10_000.0;
    let steering_centering_force = 2000.0;

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
        let turn_force = (vel.linvel.length() * 10.0).min(max_turn_force);
        force.torque = -vel.angvel * steering_centering_force;

        let mut acceleration = 0.0;
        force.force = Vec2::default();

        if keys.pressed(KeyCode::W) {
            acceleration = horse_power_fwd;
        }
        if keys.pressed(KeyCode::S) {
            acceleration = -horse_power_back;
        }
        if keys.pressed(KeyCode::A) {
            force.torque = turn_force;
        }
        if keys.pressed(KeyCode::D) {
            force.torque = -turn_force;
        }
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
