use bevy::{render::camera::RenderTarget, sprite::collide_aabb::collide};

use crate::util::get_cursor_pos_in_world_coord;

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
    objects: Query<(&Transform, &Sprite, &TooltipString), With<TooltipString>>,
) {
    let (mut tooltip, mut text) = tooltip.single_mut();

    let Some((camera, camera_transform)) = q_camera.iter().find(|(c,_)|c.is_active) else {return};

    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    text.sections[0].value = "".to_string();

    if let Some(world_pos) = get_cursor_pos_in_world_coord(wnd, camera_transform, camera) {
        tooltip.translation.x = world_pos.x + 10.0;
        tooltip.translation.y = world_pos.y - 10.0;
        tooltip.translation.z = 2.0;

        for (object, sprite, string) in objects.iter() {
            if collide(
                object.translation,
                Vec2 {
                    x: sprite.custom_size.unwrap().x,
                    y: sprite.custom_size.unwrap().y,
                },
                world_pos.extend(0.0),
                Vec2 { x: 1.0, y: 1.0 },
            )
            .is_some()
            {
                text.sections[0].value = string.0.clone();
            }
        }
    }
}
