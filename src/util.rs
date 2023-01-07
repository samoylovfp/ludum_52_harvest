use super::*;
use std::io::Cursor;
use image::{DynamicImage, ImageBuffer};

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
    Image::from_dynamic(DynamicImage::ImageRgba8(img_buf), true)
}