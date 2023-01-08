use crate::{
    harvester::Helium,
    terrain::{TerrainMarker, TerrainSprite},
    tooltip::TooltipString,
    util::image_from_aseprite,
    AppState, HEIGHT, PIXEL_MULTIPLIER, WIDTH,
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Buggy {}

pub fn setup_buggy(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let buggy_image =
        image_from_aseprite(include_bytes!("../assets/spritebuggy3.aseprite"), "Layer 1");
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
        Helium(0),
        TooltipString("Helium amount: 0".to_string()),
        TerrainMarker,
    ));
}

#[allow(clippy::type_complexity)]
pub fn buggy_movement_and_control(
    mut buggy: Query<(&Velocity, &mut ExternalForce, &Transform), (With<Buggy>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, (With<TerrainMarker>, With<Camera2d>)>,
    keys: Res<Input<KeyCode>>,
    state: Res<State<AppState>>,
    terrain: Query<&Sprite, With<TerrainSprite>>,
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
        if state.current() == &AppState::Terrain {
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
        let terrain = terrain.single().custom_size.unwrap();
        camera.get_single_mut().unwrap().translation.x = if pos.x < -(terrain.x / 2.0 - WIDTH / 2.0)
        {
            -(terrain.x / 2.0 - WIDTH / 2.0)
        } else if pos.x > (terrain.x / 2.0 - WIDTH / 2.0) {
            terrain.x / 2.0 - WIDTH / 2.0
        } else {
            pos.x
        };
        camera.get_single_mut().unwrap().translation.y =
            if pos.y < -(terrain.y / 2.0 - HEIGHT / 2.0) {
                -(terrain.y / 2.0 - HEIGHT / 2.0)
            } else if pos.y > (terrain.y / 2.0 - HEIGHT / 2.0) {
                terrain.y / 2.0 - HEIGHT / 2.0
            } else {
                pos.y
            };
        camera.get_single_mut().unwrap().translation.z = 100.0;
    }
}
