use super::*;
use bevy::render::{render_resource::SamplerDescriptor, texture::ImageSampler};
use image::{DynamicImage, ImageBuffer};
use std::io::Cursor;

pub fn image_from_aseprite(ase_bytes: &[u8], layer_name: &str) -> Image {
    image_from_aseprite_layer_name_frame(ase_bytes, layer_name, 0)
}

pub fn image_from_aseprite_layer_name_frame(
    ase_bytes: &[u8],
    layer_name: &str,
    frame: u32,
) -> Image {
    let image = asefile::AsepriteFile::read(Cursor::new(ase_bytes))
        .expect("valid aseprite")
        .layer_by_name(layer_name)
        .expect("layer name in the aseprite file")
        .frame(frame)
        .image();

    bevy_image_from_ase_image(image)
}

pub fn bevy_image_from_ase_image(image: old_image::RgbaImage) -> Image {
    let img_buf = ImageBuffer::from_raw(image.width(), image.height(), image.into_raw())
        .expect("size of containers to match");
    let mut image = Image::from_dynamic(DynamicImage::ImageRgba8(img_buf), true);
    // Disable texture filtering
    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: bevy::render::render_resource::FilterMode::Nearest,
        min_filter: bevy::render::render_resource::FilterMode::Nearest,
        ..default()
    });
    image
}

pub fn get_cursor_pos_in_world_coord(
    wnd: &Window,
    camera_transform: &GlobalTransform,
    camera: &Camera,
) -> Option<Vec2> {
    let screen_pos = wnd.cursor_position()?;

    let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    Some(world_pos.truncate())
}

#[derive(Resource)]
pub struct TerrainAssetHandlers {
    // 0 - red, 1 - yellow, 2 - green
    pub center_terrain_lamps: [(Handle<Image>, Vec2); 3],
}

#[derive(Resource)]
pub struct PanelAssetHandlers {
    // 0 - red, 1 - yellow, 2 - green
    pub center_icon: [(Handle<Image>, Vec2); 3],
}

pub fn load_assets(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let center_terrain_lamps_bytes = include_bytes!("../assets/spritecenter1.aseprite");

    commands.insert_resource(TerrainAssetHandlers {
        center_terrain_lamps: ["red", "yellow", "green"].map(|layer_name| {
            let img =
                image_from_aseprite_layer_name_frame(center_terrain_lamps_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
    });

    let center_icon_bytes = include_bytes!("../assets/iconcenter3.aseprite");

    commands.insert_resource(PanelAssetHandlers {
        center_icon: ["red", "yellow", "green"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(center_icon_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
    });
}
