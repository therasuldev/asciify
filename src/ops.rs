use image::{imageops, GrayImage, Luma};

/// Converts a flat `f32` buffer (row-major, `y * width + x`) into 8-bit grayscale.
pub fn to_gray_u8(values: &[f32], width: u32, height: u32) -> GrayImage {
    GrayImage::from_fn(width, height, |x, y| {
        Luma([values[(y * width + x) as usize]
            .clamp(0.0, 255.0)
            .round() as u8])
    })
}

/// Runs a Gaussian blur over an `f32` buffer and returns it as `f32` again.
pub fn blur(values: &[f32], width: u32, height: u32, sigma: f32) -> Vec<f32> {
    imageops::blur(&to_gray_u8(values, width, height), sigma.max(0.5))
        .pixels()
        .map(|p| p[0] as f32)
        .collect()
}

/// Value at the given percentile of the sample buffer (sorts it in place).
///
/// `total_cmp` is used: for finite values it orders them exactly like
/// `partial_cmp`, but it does not panic when a NaN is present.
pub fn percentile(values: &mut [f32], p: f32) -> f32 {
    values.sort_by(f32::total_cmp);
    values[((values.len() - 1) as f32 * p) as usize]
}
