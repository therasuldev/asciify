use crate::clahe;
use crate::cli::Opts;
use crate::image::ImageData;
use crate::mask::{BackgroundMask, CHANNEL_MAX};
use crate::ops;

/// CLAHE tile grid and contrast limit.
const CLAHE_GRID: u32 = 6;
const CLAHE_CLIP: f32 = 3.0;

/// Percentiles (low/high) clipped from the ends when stretching contrast.
const LOW_PERCENTILE: f32 = 0.02;
const HIGH_PERCENTILE: f32 = 0.98;

/// Minimum width of the stretched range (guards against division by zero).
const MIN_SPAN: f32 = 1.0;

/// Unsharp mask that brings out fine detail: `v * k - blur * (k - 1)`.
const SHARPEN_SIGMA: f32 = 2.0;
const SHARPEN_AMOUNT: f32 = 1.6;
const SHARPEN_BLUR_WEIGHT: f32 = 0.6;

/// Image with its grayscale cleaned up and contrast corrected.
pub struct ProcessedImage {
    pub values: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

/// Brings the image into a state ready for glyph conversion:
/// local contrast → background mask → global contrast → sharpen → fade background.
pub fn process(image: &ImageData, opts: &Opts) -> ProcessedImage {
    let (width, height) = (image.width, image.height);

    let equalized = local_contrast(&image.gray, width, height);
    let background = BackgroundMask::resolve(image, opts, &equalized);

    let mut values = stretch_contrast(&background.mix, &background, opts);
    sharpen(&mut values, width, height);
    fade_background(&mut values, &background, opts);

    ProcessedImage {
        values,
        width,
        height,
    }
}

/// Raises local contrast with CLAHE and returns the result as an `f32` buffer.
fn local_contrast(gray: &[f32], width: u32, height: u32) -> Vec<f32> {
    clahe::clahe(
        &ops::to_gray_u8(gray, width, height),
        CLAHE_GRID,
        CLAHE_CLIP,
    )
    .pixels()
    .map(|p| p[0] as f32)
    .collect()
}

/// Stretches contrast using only the inside of the subject (2%..98%) and applies gamma.
fn stretch_contrast(mix: &[f32], background: &BackgroundMask, opts: &Opts) -> Vec<f32> {
    let mut inside: Vec<f32> = mix
        .iter()
        .zip(&background.mask)
        .filter(|(_, m)| **m > background.threshold)
        .map(|(value, _)| *value)
        .collect();

    if inside.is_empty() {
        inside = mix.to_vec();
    }

    let low = ops::percentile(&mut inside, LOW_PERCENTILE);
    let high = ops::percentile(&mut inside, HIGH_PERCENTILE);
    let span = (high - low).max(MIN_SPAN);

    // On a dark theme bright means a dense glyph, on a light theme it is the other
    // way round. That is why gamma is inverted in light mode, otherwise the light
    // tones of the subject would turn into a dense "@" blob.
    let gamma = if opts.light { 1.0 / opts.gamma } else { opts.gamma };

    mix.iter()
        .map(|x| ((x - low) / span).clamp(0.0, 1.0).powf(gamma) * CHANNEL_MAX)
        .collect()
}

/// Light sharpening: `1.6 * v - 0.6 * blur(v)`.
fn sharpen(values: &mut [f32], width: u32, height: u32) {
    let blurred = ops::blur(values, width, height, SHARPEN_SIGMA);

    for (value, blurred) in values.iter_mut().zip(&blurred) {
        *value = SHARPEN_AMOUNT * *value - SHARPEN_BLUR_WEIGHT * blurred;
    }
}

/// Fades the background: towards black on a dark theme, towards white on a light one.
fn fade_background(values: &mut [f32], background: &BackgroundMask, opts: &Opts) {
    let background_value = if opts.light { CHANNEL_MAX } else { 0.0 };

    for (value, fade) in values.iter_mut().zip(&background.fade) {
        *value = background_value
            + (value.clamp(0.0, CHANNEL_MAX) - background_value) * *fade;
    }
}
