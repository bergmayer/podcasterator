use image::{imageops::FilterType, ImageReader};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const TARGET_SIZE: u32 = 1400;
const JPEG_QUALITY: u8 = 90;

/// Convert and resize an image to 1400x1400 JPEG
pub fn process_artwork(src_path: &Path, dst_path: &Path) -> Result<(), String> {
    // Load the source image
    let img = ImageReader::open(src_path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Resize to 1400x1400 using Lanczos3 filter
    let resized = img.resize_exact(TARGET_SIZE, TARGET_SIZE, FilterType::Lanczos3);

    // Convert to RGB for JPEG (no alpha channel)
    let rgb_img = resized.to_rgb8();

    // Save as JPEG
    let file =
        File::create(dst_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY);
    rgb_img
        .write_with_encoder(encoder)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    Ok(())
}
