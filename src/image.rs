use std::error::Error;

use image::{imageops, RgbaImage};

use crate::cli::Opts;

/// Per-pixel opacity cut-off: anything below this alpha counts as transparent.
const OPAQUE_THRESHOLD: u8 = 250;

/// Alpha cut-off (0..1) for deciding which pixels belong to the subject.
const MASK_THRESHOLD: f32 = 0.5;

/// Grayscale and alpha channels extracted from the input image.
pub struct ImageData {
    /// Grayscale (0..255), row-major.
    pub gray: Vec<f32>,
    /// Alpha (0..1), row-major. All 1.0 when the image has no transparency.
    pub alpha: Vec<f32>,
    pub has_alpha: bool,
    pub width: u32,
    pub height: u32,
}

pub fn load(opts: &Opts) -> Result<ImageData, Box<dyn Error>> {
    let rgba = load_rgba(opts)?;
    let (width, height) = rgba.dimensions();

    let mut gray = grayscale(&rgba);
    let (alpha, has_alpha) = alpha_channel(&rgba);

    if has_alpha {
        fill_transparent(&mut gray, &alpha);
    }

    Ok(ImageData {
        gray,
        alpha,
        has_alpha,
        width,
        height,
    })
}

/// Opens the file and crops the rectangle given by `--crop`, if any.
fn load_rgba(opts: &Opts) -> Result<RgbaImage, Box<dyn Error>> {
    let rgba = image::open(&opts.input)?.to_rgba8();

    let Some((x1, y1, x2, y2)) = opts.crop else {
        return Ok(rgba);
    };

    let (width, height) = rgba.dimensions();

    if x2 > width || y2 > height {
        return Err(
            format!("--crop şəkildən kənara çıxır (şəkil {width}x{height})").into(),
        );
    }

    Ok(imageops::crop_imm(&rgba, x1, y1, x2 - x1, y2 - y1).to_image())
}

/// Rec.601 luma conversion.
fn grayscale(rgba: &RgbaImage) -> Vec<f32> {
    rgba.pixels()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect()
}

/// Scales the alpha channel to 0..1; returns all 1.0 when there is no transparency.
fn alpha_channel(rgba: &RgbaImage) -> (Vec<f32>, bool) {
    let has_alpha = rgba.pixels().any(|p| p[3] < OPAQUE_THRESHOLD);

    if !has_alpha {
        let (width, height) = rgba.dimensions();
        return (vec![1.0; (width * height) as usize], false);
    }

    (
        rgba.pixels().map(|p| p[3] as f32 / 255.0).collect(),
        true,
    )
}

/// Replaces transparent pixels with the mean tone of the subject so the CLAHE
/// histograms are not skewed by the empty background.
fn fill_transparent(gray: &mut [f32], alpha: &[f32]) {
    let (sum, count) = gray
        .iter()
        .zip(alpha)
        .filter(|(_, a)| **a > MASK_THRESHOLD)
        .fold((0.0, 0usize), |(sum, count), (g, _)| {
            (sum + g, count + 1)
        });

    let mean = if count > 0 {
        sum / count as f32
    } else {
        128.0
    };

    for (g, a) in gray.iter_mut().zip(alpha) {
        if *a <= MASK_THRESHOLD {
            *g = mean;
        }
    }
}
