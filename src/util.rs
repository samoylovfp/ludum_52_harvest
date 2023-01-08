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

    let window_size = Vec2::new(wnd.width(), wnd.height());
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    Some(world_pos.truncate())
}

pub type ImgHWithSize = (Handle<Image>, Vec2);

#[derive(Resource)]
pub struct TerrainAssetHandlers {
    // 0 - red, 1 - yellow, 2 - green
    pub center_terrain_lamps: [ImgHWithSize; 3],
    pub harvester: ImgHWithSize,
    pub center: ImgHWithSize,
	// 0 - button, 1 - red lamp, 2 - green lamp
	pub map_button: [ImgHWithSize; 3],
}

#[derive(Resource)]
pub struct PanelAssetHandlers {
    // 0 - red, 1 - yellow, 2 - green
    pub center_icon: [ImgHWithSize; 3],
    pub buggy_icon: ImgHWithSize,
    pub harv_icon: ImgHWithSize,
    pub ship: ImgHWithSize,
	// 3 frames animation
    pub space: [ImgHWithSize; 3],
	// 6 slots, each has 0 - not set, 1 - green, 2 - yellow, 3 - red
    pub harv_slots: [[ImgHWithSize; 4]; 6],
    pub exit: ImgHWithSize,
	// 0 - button, 1 - writing gray, 2 - writing green
    pub harvester_button: [ImgHWithSize; 3],
	// 0 - button, 1 - writing gray, 2 - writing green
    pub tank_button: [ImgHWithSize; 3],
    pub helium_level: ImgHWithSize,
	// 5 tanks
    pub tanks: [ImgHWithSize; 5],
}

fn img_handle_and_size_from_bytes(
    b: &[u8],
    layer_name: &str,
    textures: &mut ResMut<Assets<Image>>,
) -> ImgHWithSize {
    let img = image_from_aseprite_layer_name_frame(b, layer_name, 0);
    let size = img.size();
    (textures.add(img), size * PIXEL_MULTIPLIER)
}

pub fn load_assets(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let center_terrain_lamps_bytes = include_bytes!("../assets/spritecenter1.aseprite");
	let button_bytes = include_bytes!("../assets/spritebutton2.aseprite");

    commands.insert_resource(TerrainAssetHandlers {
        center_terrain_lamps: ["red", "yellow", "green"].map(|layer_name| {
            img_handle_and_size_from_bytes(center_terrain_lamps_bytes, layer_name, &mut textures)
        }),
        harvester: img_handle_and_size_from_bytes(
            include_bytes!("../assets/spriteharvester1.aseprite"),
            "Layer 1",
            &mut textures,
        ),
        center: img_handle_and_size_from_bytes(
            include_bytes!("../assets/spritecenter1.aseprite"),
            "base",
            &mut textures,
        ),
		map_button: ["buttonup", "red", "green"].map(|layer_name| {
            img_handle_and_size_from_bytes(button_bytes, layer_name, &mut textures)
        }),
    });

    let center_icon_bytes = include_bytes!("../assets/iconcenter3.aseprite");
    let panel_bytes = include_bytes!("../assets/spritepanel8.aseprite");

    commands.insert_resource(PanelAssetHandlers {
        center_icon: ["red", "yellow", "green"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(center_icon_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
        buggy_icon: img_handle_and_size_from_bytes(
            include_bytes!("../assets/iconbuggy1.aseprite"),
            "Layer 1",
            &mut textures,
        ),
        harv_icon: img_handle_and_size_from_bytes(
            include_bytes!("../assets/iconharvest1.aseprite"),
            "Layer 1",
            &mut textures,
        ),
        ship: img_handle_and_size_from_bytes(
            include_bytes!("../assets/iconship1.aseprite"),
            "Layer 1",
            &mut textures,
        ),
        space: ["space1", "space2", "space3"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
        harv_slots: [
            ["harv1off", "harv1green", "harv1yellow", "harv1red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
            ["harv2off", "harv2green", "harv2yellow", "harv2red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
            ["harv3off", "harv3green", "harv3yellow", "harv3red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
            ["harv4off", "harv4green", "harv4yellow", "harv4red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
            ["harv5off", "harv5green", "harv5yellow", "harv5red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
            ["harv6off", "harv6green", "harv6yellow", "harv6red"].map(|layer_name| {
                let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
                let size = img.size();
                (textures.add(img), size * PIXEL_MULTIPLIER)
            }),
        ],
        exit: img_handle_and_size_from_bytes(
            panel_bytes,
            "exitup",
            &mut textures,
        ),
        harvester_button: ["harvesterup", "harvesteroff", "harvestergreen"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
        tank_button: ["tankup", "tankoff", "tankgreen"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
        helium_level: img_handle_and_size_from_bytes(
            panel_bytes,
            "he3",
            &mut textures,
        ),
        tanks: ["tank1", "tank2", "tank3", "tank4", "tank5"].map(|layer_name| {
            let img = image_from_aseprite_layer_name_frame(panel_bytes, layer_name, 0);
            let size = img.size();
            (textures.add(img), size * PIXEL_MULTIPLIER)
        }),
    });
}
