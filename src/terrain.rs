use crate::{
    buggy::{buggy_movement_and_control, setup_buggy, Buggy},
    harvester::{
        move_harvesters, BreakTime, Center, HarvesterState, Helium, StorageHelium, TotalHarvesters,
        MAX_HELIUM, Tanks,
    },
    tooltip::TooltipString,
    util::{image_from_aseprite, TerrainAssetHandlers},
    AppState, CELL_SIZE_TERRAIN, HEIGHT, PIXEL_MULTIPLIER, WIDTH,
};
use bevy::{prelude::*, render::camera::RenderTarget, sprite::collide_aabb::collide};
use bevy_rapier2d::prelude::*;
use once_cell::sync::OnceCell;
use rand::{thread_rng, Rng};

pub const COLLECT_DISTANCE: f32 = 500.0;
pub const TERRAIN_SIZE: (f32, f32) = (440.0 * PIXEL_MULTIPLIER, 320.0 * PIXEL_MULTIPLIER);
pub const MAX_HELIUM_STORAGE: usize = 20;
pub const HELIUM_TO_BUILD_HARVESTER: usize = MAX_HELIUM_STORAGE / 2;

#[derive(Component)]
pub struct TerrainMarker;

#[derive(Component)]
pub struct TerrainSprite;

#[derive(Component)]
pub struct MapButton;

#[derive(Component)]
pub struct Base;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_exit(AppState::Start)
                .with_system(setup_terrain)
                .with_system(setup_buggy),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Terrain)
                .with_system(mouse_clicks)
                .with_system(update_button)
				.with_system(update_base),
        )
        .add_system_set(SystemSet::on_enter(AppState::Terrain).with_system(enable_terrain_cam))
        .add_system(move_harvesters)
        .add_system(buggy_movement_and_control)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(12.0));
        // .add_plugin(RapierDebugRenderPlugin::default());
    }
}

fn setup_terrain(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut phys: ResMut<RapierConfiguration>,
    terrain_assets: Res<TerrainAssetHandlers>,
) {
    //FIXME filsam: reduce boilerplate
    static TERRAIN_IMAGE_CELL: OnceCell<Image> = OnceCell::new();
    static TERRAIN_TEXTURE_HANDLE_CELL: OnceCell<Handle<Image>> = OnceCell::new();

    let terrain_image = TERRAIN_IMAGE_CELL.get_or_init(|| {
        image_from_aseprite(include_bytes!("../assets/spritemap5.aseprite"), "Layer 1")
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
        .insert(TerrainSprite)
        .insert(TerrainMarker);
    commands.spawn((Camera2dBundle::default(), TerrainMarker));
    phys.gravity = Vec2 { x: 0.0, y: 0.0 };

    let collider_width = 100.0;
    let offset = 98.0;
    let parameters = vec![
        vec![0.0, size.y / 2.0 + offset, size.x, collider_width],
        vec![0.0, -(size.y / 2.0 + offset), size.x, collider_width],
        vec![size.x / 2.0 + offset, 0.0, collider_width, size.y],
        vec![-(size.x / 2.0 + offset), 0.0, collider_width, size.y],
    ];

    for collider in parameters {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite { ..default() },
                transform: Transform {
                    translation: Vec3::new(collider[0], collider[1], 1.0),
                    ..Default::default()
                },
                ..default()
            })
            .insert((RigidBody::Fixed, Collider::cuboid(collider[2], collider[3])))
            .insert(TerrainMarker);
    }

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(terrain_assets.map_button[0].1),
                ..default()
            },
            texture: terrain_assets.map_button[0].0.clone(),
            ..default()
        })
        .insert(MapButton)
		.insert(TooltipString("Open info panel".to_string()))
        .insert(TerrainMarker);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                // color: Color::RED,
                custom_size: Some(Vec2::new(
                    CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER,
                    CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER,
                )),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    TERRAIN_SIZE.0 / 2.0 - CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
                    CELL_SIZE_TERRAIN * PIXEL_MULTIPLIER / 2.0,
                    -100.0,
                ),
                ..Default::default()
            },
            ..default()
        })
        .insert(Base)
        .insert(TooltipString("Base".to_string()))
        .insert(TerrainMarker);

    commands.insert_resource(TotalHarvesters(0));
    commands.insert_resource(StorageHelium(MAX_HELIUM_STORAGE - 1));
	commands.insert_resource(Tanks(0));
}

fn enable_terrain_cam(
    mut cam: Query<&mut Camera, With<TerrainMarker>>,
    mut panel_cam: Query<&mut Camera, Without<TerrainMarker>>,
) {
    cam.for_each_mut(|mut c| c.is_active = true);
    panel_cam.for_each_mut(|mut c| c.is_active = false);
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn mouse_clicks(
    mut buggy: Query<(&Transform, &mut Helium, &mut TooltipString), (With<Buggy>, Without<Center>)>,
    mut centers: Query<
        (
            &Transform,
            &Sprite,
            &mut Helium,
            &mut HarvesterState,
            &mut BreakTime,
        ),
        (With<Center>, Without<Buggy>),
    >,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut buttons: ResMut<Input<MouseButton>>,
    mut app_state: ResMut<State<AppState>>,
    map_button: Query<(&Transform, &Sprite), With<MapButton>>,
	base: Query<(&Transform, &Sprite), With<Base>>,
	mut storage_total: ResMut<StorageHelium>
) {
    let (buggy, mut storage, mut buggy_string) = buggy.single_mut();
    let Some((camera, camera_transform)) = q_camera.iter().find(|(c,_)|c.is_active) else {return};

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(screen_pos) = wnd.cursor_position() {
            let window_size = Vec2::new(wnd.width(), wnd.height());
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            let (map_button, button_sprite) = map_button.single();

            if collide(
                map_button.translation,
                Vec2 {
                    x: button_sprite.custom_size.unwrap().x,
                    y: button_sprite.custom_size.unwrap().y,
                },
                world_pos,
                Vec2 { x: 1.0, y: 1.0 },
            )
            .is_some()
            {
                app_state.set(AppState::Panel).unwrap();
                buttons.clear();
                return;
            }

			let (base, base_sprite) = base.single();

			if collide(
                base.translation,
                Vec2 {
                    x: base_sprite.custom_size.unwrap().x,
                    y: base_sprite.custom_size.unwrap().y,
                },
                world_pos,
                Vec2 { x: 1.0, y: 1.0 },
            )
            .is_some()
            {
                storage_total.0 += storage.0;
				storage.0 = 0;
				if storage_total.0 > MAX_HELIUM_STORAGE {
					storage.0 += storage_total.0 - MAX_HELIUM_STORAGE;
					storage_total.0 = MAX_HELIUM_STORAGE;
				}
				buggy_string.0 = format!("Helium amount: {}", storage.0);
                return;
            }

            for (center, sprite, mut helium, mut state, mut breaktime) in centers.iter_mut() {
                if collide(
                    center.translation,
                    Vec2 {
                        x: sprite.custom_size.unwrap().x,
                        y: sprite.custom_size.unwrap().y,
                    },
                    world_pos,
                    Vec2 { x: 1.0, y: 1.0 },
                )
                .is_some()
                    && center.translation.distance(buggy.translation) <= COLLECT_DISTANCE
                {
                    match *state {
                        HarvesterState::Work => {
                            storage.0 += helium.0;
                            helium.0 = 0;
                        }
                        HarvesterState::Full => {
                            storage.0 += helium.0;
                            helium.0 = 0;
                            *state = HarvesterState::Work;
                        }
                        HarvesterState::Broken => {
                            let mut rng = thread_rng();
                            breaktime.0 = rng.gen_range(100..1000);
                            if helium.0 == MAX_HELIUM {
                                *state = HarvesterState::Full;
                            } else {
                                *state = HarvesterState::Work;
                            }
                        }
                    };
                    buggy_string.0 = format!("Helium amount: {}", storage.0);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_button(
    camera: Query<&Transform, (With<TerrainMarker>, With<Camera2d>, Without<MapButton>)>,
    mut button: Query<(&mut Transform, &mut Handle<Image>), With<MapButton>>,
    centers: Query<&HarvesterState, With<Center>>,
    terrain_assets: Res<TerrainAssetHandlers>,
) {
    let camera = camera.single().translation;
    let (mut button, mut sprite) = button.single_mut();
    button.translation.x = camera.x - WIDTH / 2.0 + 40.0;
    button.translation.y = camera.y + HEIGHT / 2.0 - 50.0;
    button.translation.z = 3.0;

    *sprite = terrain_assets.map_button[0].0.clone();
    for center in centers.iter() {
        if !matches!(*center, HarvesterState::Work) {
            *sprite = terrain_assets.map_button[1].0.clone();
        }
    }
}

fn update_base(mut base: Query<&mut TooltipString, With<Base>>, storage_total: Res<StorageHelium>) {
	let mut string = base.single_mut();
	string.0 = format!("Helium:\n{}/{}", storage_total.0, MAX_HELIUM_STORAGE);
}
