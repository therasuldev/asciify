// asciify: converts a portrait photo into ASCII art and saves the result as a PNG.
//
// Usage:
//   asciify image.png -o ascii.png --crop 840,940,1500,1760 --cols 100 --gamma 1.6 --soft-bg
//
// About the background:
//   * If the image has a transparent background (alpha), it is used as the mask
//     automatically (cleanest result).
//   * Otherwise, when --soft-bg is passed, the background is blurred and faded out softly.
//   * With neither, the image is converted as it is.
//
// Modules:
//   cli         — reading the options
//   image       — opening the file, cropping, grayscale + alpha
//   clahe       — local contrast (CLAHE)
//   ops         — shared buffer helpers (to_gray_u8, blur, percentile)
//   mask        — background mask (alpha / --soft-bg / disabled)
//   processing  — contrast, sharpening, background fade
//   font        — font and glyph cell metrics
//   ascii       — pixels → glyphs
//   render      — glyphs → PNG

mod ascii;
mod clahe;
mod cli;
mod font;
mod image;
mod mask;
mod ops;
mod processing;
mod render;

use std::error::Error;
use std::process;

fn run() -> Result<(), Box<dyn Error>> {
    let opts = cli::parse_args()?;
    let font = font::Font::load(opts.font_px)?;

    let image = image::load(&opts)?;
    let processed = processing::process(&image, &opts);
    let art = ascii::convert(&processed, &opts, &font);

    render::save(&art, &opts, &font)?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Xəta: {e}");
        process::exit(1);
    }
}
