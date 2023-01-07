use bevy::{render::camera::RenderTarget, sprite::collide_aabb::collide};

use super::*;

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipString(pub String);

pub fn spawn_tooltip(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = include_bytes!("../assets/PublicPixel-z84yD.ttf");
    let font_handle = fonts.add(Font::try_from_bytes(font.to_vec()).expect("valid font"));

    let text_style = TextStyle {
        font: font_handle,
        font_size: 15.0,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment::TOP_LEFT;

    commands.spawn((
        Text2dBundle {
            text: Text::from_section("translation", text_style.clone())
                .with_alignment(text_alignment),
            ..default()
        },
        Tooltip,
    ));
}

pub fn update_tooltip(
    mut tooltip: Query<(&mut Transform, &mut Text), (With<Tooltip>, Without<TooltipString>)>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    objects: Query<(&Transform, &TooltipString), With<TooltipString>>,
) {
    let (mut tooltip, mut text) = tooltip.single_mut();

    let (camera, camera_transform) = q_camera.single();

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    let mut world_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    if let Some(screen_pos) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        // let world_pos: Vec2 = world_pos.truncate();

        tooltip.translation.x = world_pos.x + 10.0;
        tooltip.translation.y = world_pos.y - 10.0;
        tooltip.translation.z = 2.0;
        dbg!(tooltip.translation);
    }

    text.sections[0].value = "".to_string();

    for (object, string) in objects.iter() {
        if collide(
            object.translation,
            Vec2 {
                x: 15.0 * PIXEL_MULTIPLIER,
                y: 15.0 * PIXEL_MULTIPLIER,
            },
            world_pos,
            Vec2 { x: 1.0, y: 1.0 },
        )
        .is_some()
        {
            text.sections[0].value = string.0.clone();
        }
    }
}