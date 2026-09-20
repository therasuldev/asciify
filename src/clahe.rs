use image::{GrayImage, Luma};

/// Number of histogram bins (8-bit grayscale).
const BINS: usize = 256;

/// CLAHE: local contrast enhancement. The image is split into a `grid x grid` of
/// tiles, each tile's histogram is clipped and equalized, then bilinear
/// interpolation blends the tiles back together.
pub fn clahe(src: &GrayImage, grid: u32, clip: f32) -> GrayImage {
    let (w, h) = src.dimensions();
    let (tw, th) = (w as f32 / grid as f32, h as f32 / grid as f32);
    let mut luts = vec![[0u8; BINS]; (grid * grid) as usize];

    for ty in 0..grid {
        for tx in 0..grid {
            let x0 = (tx as f32 * tw) as u32;
            let x1 = if tx == grid - 1 {
                w
            } else {
                ((tx + 1) as f32 * tw) as u32
            };
            let y0 = (ty as f32 * th) as u32;
            let y1 = if ty == grid - 1 {
                h
            } else {
                ((ty + 1) as f32 * th) as u32
            };

            let mut hist = [0u32; BINS];
            for y in y0..y1 {
                for x in x0..x1 {
                    hist[src.get_pixel(x, y)[0] as usize] += 1;
                }
            }
            let npix = ((x1 - x0) * (y1 - y0)).max(1);
            let limit = ((clip * npix as f32 / BINS as f32) as u32).max(1);

            // clip everything above the limit, then redistribute it evenly
            let mut excess = 0u32;
            for b in hist.iter_mut() {
                if *b > limit {
                    excess += *b - limit;
                    *b = limit;
                }
            }
            let add = excess / BINS as u32;
            let mut rem = (excess % BINS as u32) as usize;
            for b in hist.iter_mut() {
                *b += add;
            }
            let step = if rem > 0 { (BINS / rem).max(1) } else { BINS };
            let mut i = 0;
            while rem > 0 && i < BINS {
                hist[i] += 1;
                rem -= 1;
                i += step;
            }

            let lut = &mut luts[(ty * grid + tx) as usize];
            let mut cum = 0u32;
            for (i, b) in hist.iter().enumerate() {
                cum += *b;
                lut[i] = ((cum as f32 * 255.0 / npix as f32).round()).min(255.0) as u8;
            }
        }
    }

    let clamp_tile = |index: i32| index.clamp(0, grid as i32 - 1) as u32;

    GrayImage::from_fn(w, h, |x, y| {
        let fx = (x as f32 + 0.5) / tw - 0.5;
        let fy = (y as f32 + 0.5) / th - 0.5;
        let (ix, iy) = (fx.floor() as i32, fy.floor() as i32);
        let (wx, wy) = (fx - ix as f32, fy - iy as f32);
        let (xa, xb, ya, yb) = (
            clamp_tile(ix),
            clamp_tile(ix + 1),
            clamp_tile(iy),
            clamp_tile(iy + 1),
        );

        let p = src.get_pixel(x, y)[0] as usize;
        let lookup = |tx: u32, ty: u32| luts[(ty * grid + tx) as usize][p] as f32;

        let top = lookup(xa, ya) * (1.0 - wx) + lookup(xb, ya) * wx;
        let bottom = lookup(xa, yb) * (1.0 - wx) + lookup(xb, yb) * wx;

        Luma([(top * (1.0 - wy) + bottom * wy).round() as u8])
    })
}
