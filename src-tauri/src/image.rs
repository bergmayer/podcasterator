use image::{imageops::FilterType, ImageReader};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const MAX_SIZE: u32 = 1400;
const JPEG_QUALITY: u8 = 90;

/// Convert and resize an image to a square JPEG suitable for podcast artwork.
/// Non-square images are center-cropped rather than stretched.
/// Images larger than 1400px are downscaled; smaller images keep their size.
pub fn process_artwork(src_path: &Path, dst_path: &Path) -> Result<(), String> {
    let img = ImageReader::open(src_path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Use the smaller dimension, capped at MAX_SIZE (don't upscale beyond source)
    let size = img.width().min(img.height()).min(MAX_SIZE);

    let resized = img.resize_to_fill(size, size, FilterType::Lanczos3);
    let rgb_img = resized.to_rgb8();

    let file =
        File::create(dst_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY);
    rgb_img
        .write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    Ok(())
}
