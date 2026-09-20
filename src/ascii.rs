use image::imageops;

use crate::cli::Opts;
use crate::font::Font;
use crate::ops;
use crate::processing::ProcessedImage;

/// Glyph ramp: dense → sparse. Used as-is on a light background, reversed on a dark one.
pub const RAMP: &str = "@%#*+=-:. ";

/// Averaging filter when downscaling — nearest-neighbour made the edges jagged.
const RESIZE_FILTER: imageops::FilterType = imageops::FilterType::Triangle;

/// Tone range of a single pixel.
const TONE_RANGE: usize = 255;

/// Finished ASCII art in the form of a glyph grid.
pub struct AsciiImage {
    pub lines: Vec<String>,
    pub cols: u32,
    pub rows: u32,
}

/// Converts a processed image into a glyph grid.
pub fn convert(image: &ProcessedImage, opts: &Opts, font: &Font) -> AsciiImage {
    let rows = row_count(image, opts, font);

    let small = imageops::resize(
        &ops::to_gray_u8(&image.values, image.width, image.height),
        opts.cols,
        rows,
        RESIZE_FILTER,
    );

    let ramp = ramp(opts.light);
    let last = ramp.len() - 1;

    let lines = (0..rows)
        .map(|y| {
            (0..opts.cols)
                .map(|x| {
                    let tone = small.get_pixel(x, y)[0] as usize;
                    ramp[tone * last / TONE_RANGE]
                })
                .collect()
        })
        .collect();

    AsciiImage {
        lines,
        cols: opts.cols,
        rows,
    }
}

/// Row count: the cell's width/height ratio is folded in so that the PNG keeps the
/// original aspect ratio (a monospace cell is taller than it is wide).
fn row_count(image: &ProcessedImage, opts: &Opts, font: &Font) -> u32 {
    let image_aspect = image.height as f32 / image.width as f32;
    let cell_aspect = font.cell_width / font.cell_height;

    (opts.cols as f32 * image_aspect * cell_aspect)
        .round()
        .max(1.0) as u32
}

fn ramp(light: bool) -> Vec<char> {
    let chars: Vec<char> = RAMP.chars().collect();

    if light {
        chars
    } else {
        chars.into_iter().rev().collect()
    }
}
