use std::error::Error;

use ab_glyph::{point, Font as _, ScaleFont};
use image::{Rgb, RgbImage};

use crate::ascii::AsciiImage;
use crate::cli::Opts;
use crate::font::Font;

const DARK_BACKGROUND: Rgb<u8> = Rgb([13, 17, 23]);
const DARK_FOREGROUND: Rgb<u8> = Rgb([230, 237, 243]);
const LIGHT_BACKGROUND: Rgb<u8> = Rgb([255, 255, 255]);
const LIGHT_FOREGROUND: Rgb<u8> = Rgb([20, 20, 20]);

/// A space is never drawn — it is the "nothing" that matches the empty background.
const BLANK: char = ' ';

const RGB_CHANNELS: usize = 3;

/// Draws the ASCII art as a PNG and writes it to `opts.output`.
pub fn save(ascii: &AsciiImage, opts: &Opts, font: &Font) -> Result<(), Box<dyn Error>> {
    let (background, foreground) = palette(opts.light);
    let padding = opts.font_px.round() as u32;

    let image_width = (ascii.cols as f32 * font.cell_width).ceil() as u32 + 2 * padding;
    let image_height = (ascii.rows as f32 * font.cell_height) as u32 + 2 * padding;

    let mut canvas = RgbImage::from_pixel(image_width, image_height, background);

    for (row, line) in ascii.lines.iter().enumerate() {
        draw_line(
            &mut canvas,
            font,
            padding as f32,
            padding as f32 + row as f32 * font.cell_height,
            line,
            foreground,
        );
    }

    canvas.save(&opts.output)?;

    eprintln!(
        "Hazırdır: {} ({}x{} px, {} sütun x {} sətir)",
        opts.output, image_width, image_height, ascii.cols, ascii.rows
    );

    Ok(())
}

fn palette(light: bool) -> (Rgb<u8>, Rgb<u8>) {
    if light {
        (LIGHT_BACKGROUND, LIGHT_FOREGROUND)
    } else {
        (DARK_BACKGROUND, DARK_FOREGROUND)
    }
}

/// Draws one line of text onto the canvas (via ab_glyph, anti-aliasing included).
fn draw_line(
    canvas: &mut RgbImage,
    font: &Font,
    x0: f32,
    y0: f32,
    text: &str,
    foreground: Rgb<u8>,
) {
    let scaled = font.font.as_scaled(font.scale);
    let baseline = y0 + scaled.ascent();
    let (width, height) = canvas.dimensions();

    let mut x = x0;

    for character in text.chars() {
        let glyph_id = font.font.glyph_id(character);

        if character != BLANK {
            let glyph = glyph_id.with_scale_and_position(font.scale, point(x, baseline));

            if let Some(outline) = font.font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();

                outline.draw(|gx, gy, coverage| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;

                    if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                        let pixel = canvas.get_pixel_mut(px as u32, py as u32);

                        for channel in 0..RGB_CHANNELS {
                            pixel[channel] = (pixel[channel] as f32 * (1.0 - coverage)
                                + foreground[channel] as f32 * coverage)
                                .round() as u8;
                        }
                    }
                });
            }
        }

        x += scaled.h_advance(glyph_id);
    }
}
