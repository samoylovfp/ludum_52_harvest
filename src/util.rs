use super::*;
use bevy::render::{render_resource::SamplerDescriptor, texture::ImageSampler};
use image::{DynamicImage, ImageBuffer};
use std::io::Cursor;

pub fn image_from_aseprite(ase_bytes: &[u8]) -> Image {
    let image = asefile::AsepriteFile::read(Cursor::new(ase_bytes))
        .expect("valid aseprite")
        .layers()
        .next()
        .expect("at least one layer")
        .frame(0)
        .image();

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
