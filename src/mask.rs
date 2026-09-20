use crate::cli::Opts;
use crate::image::ImageData;
use crate::ops;

/// Upper bound of the 8-bit tone range.
pub(crate) const CHANNEL_MAX: f32 = 255.0;

/// Blur radii are expressed relative to the image width.
const ALPHA_BLUR_FACTOR: f32 = 0.004;
const SUBJECT_BLUR_FACTOR: f32 = 0.027;
const MASK_BLUR_FACTOR: f32 = 0.068;

/// The ellipse used by `--soft-bg`: centre and radii relative to width/height.
const ELLIPSE_CENTER: (f32, f32) = (0.52, 0.5);
const ELLIPSE_RADIUS: (f32, f32) = (0.45, 0.50);

/// How far the background is faded out at most.
const FADE_FLOOR: f32 = 0.10;
const FADE_RANGE: f32 = 0.90;

/// Which pixels count as "inside the subject".
const ALPHA_THRESHOLD: f32 = 0.5;
const SOFT_BG_THRESHOLD: f32 = 0.6;
const NO_BACKGROUND_THRESHOLD: f32 = 0.0;

/// The mask describing where the background is, plus the matching fade factors.
///
/// There are three modes:
///   * the image has transparency — the alpha channel is used as the mask directly
///     (cleanest result);
///   * `--soft-bg` was passed — an ellipse centre with a strong blur gives a soft
///     transition;
///   * neither — background fading is disabled entirely.
pub(crate) struct BackgroundMask {
    /// Tone buffer the contrast is computed from (background-blended variant).
    pub(crate) mix: Vec<f32>,
    /// 1.0 = subject, 0.0 = background.
    pub(crate) mask: Vec<f32>,
    /// Fade factor derived from the mask (1.0 = no fading at all).
    pub(crate) fade: Vec<f32>,
    /// Pixels with `mask > threshold` feed the contrast statistics.
    pub(crate) threshold: f32,
}

impl BackgroundMask {
    pub(crate) fn resolve(image: &ImageData, opts: &Opts, gray: &[f32]) -> Self {
        if image.has_alpha {
            return Self::from_alpha(image, gray);
        }

        if opts.soft_bg {
            return Self::from_soft_bg(image, gray);
        }

        Self::disabled(image, gray)
    }

    fn from_alpha(image: &ImageData, gray: &[f32]) -> Self {
        let (width, height) = (image.width, image.height);

        let alpha_255: Vec<f32> = image.alpha.iter().map(|a| a * CHANNEL_MAX).collect();

        let mask = normalize(ops::blur(
            &alpha_255,
            width,
            height,
            ALPHA_BLUR_FACTOR * width as f32,
        ));

        Self {
            mix: gray.to_vec(),
            fade: mask.clone(),
            mask,
            threshold: ALPHA_THRESHOLD,
        }
    }

    fn from_soft_bg(image: &ImageData, gray: &[f32]) -> Self {
        let (width, height) = (image.width, image.height);
        let width_f = width as f32;

        let mask = normalize(ops::blur(
            &ellipse(width, height),
            width,
            height,
            MASK_BLUR_FACTOR * width_f,
        ));

        // Blurred copy that erases the background texture.
        let blurred = ops::blur(gray, width, height, SUBJECT_BLUR_FACTOR * width_f);

        let mix = gray
            .iter()
            .zip(&blurred)
            .zip(&mask)
            .map(|((g, blurred), m)| g * m + blurred * (1.0 - m))
            .collect();

        let fade = mask.iter().map(|m| FADE_FLOOR + FADE_RANGE * m).collect();

        Self {
            mix,
            mask,
            fade,
            threshold: SOFT_BG_THRESHOLD,
        }
    }

    fn disabled(image: &ImageData, gray: &[f32]) -> Self {
        let count = (image.width * image.height) as usize;

        Self {
            mix: gray.to_vec(),
            mask: vec![1.0; count],
            fade: vec![1.0; count],
            threshold: NO_BACKGROUND_THRESHOLD,
        }
    }
}

/// An ellipse centred in the image (255 = subject, 0 = background).
fn ellipse(width: u32, height: u32) -> Vec<f32> {
    let (cx, cy) = (
        width as f32 * ELLIPSE_CENTER.0,
        height as f32 * ELLIPSE_CENTER.1,
    );
    let (rx, ry) = (
        width as f32 * ELLIPSE_RADIUS.0,
        height as f32 * ELLIPSE_RADIUS.1,
    );
    let count = (width * height) as usize;

    (0..count)
        .map(|i| {
            let (x, y) = ((i as u32 % width) as f32, (i as u32 / width) as f32);
            let distance = ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2);

            if distance <= 1.0 {
                CHANNEL_MAX
            } else {
                0.0
            }
        })
        .collect()
}

/// Converts a 0..255 buffer into a 0..1 mask.
fn normalize(values: Vec<f32>) -> Vec<f32> {
    values.into_iter().map(|v| v / CHANNEL_MAX).collect()
}
