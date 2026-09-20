# asciify

Converts a photo into ASCII art and renders the result as a **PNG image**.

The output is not printed to the terminal — the glyph grid is drawn with a real
monospace font and saved as a picture, so it is high-resolution and can be shared
directly.

```
input photo  →  grayscale + local contrast  →  glyph grid  →  ASCII art PNG
```

---

## Table of contents

- [Requirements](#requirements)
- [Quick start](#quick-start)
- [Command-line reference](#command-line-reference)
- [Examples](#examples)
- [How it works](#how-it-works)
- [Background handling](#background-handling)
- [Project layout](#project-layout)
- [Design notes](#design-notes)
- [Error messages](#error-messages)

---

## Requirements

- **Rust** with an edition-2021 toolchain (developed and verified against `rustc 1.96`).
- Two crates from crates.io, resolved automatically by Cargo:

  | crate | version | used for |
  |---|---|---|
  | [`image`](https://crates.io/crates/image) | 0.25 (PNG + JPEG only) | decoding, cropping, resizing, blurring, saving |
  | [`ab_glyph`](https://crates.io/crates/ab_glyph) | 0.2 | loading the font and rasterising glyphs |

- The font `assets/DejaVuSansMono.ttf` must be present — it is embedded into the
  binary at compile time via `include_bytes!`, so the executable has no runtime
  font dependency.

## Quick start

```bash
# build and run on your own photo
cargo run --release -- photo.png -o ascii.png

# then open the result
open ascii.png
```

On macOS/Linux the binary itself is at `target/release/asciify`, so after building
once you can run it without Cargo:

```bash
cargo build --release
./target/release/asciify photo.png -o ascii.png --cols 120 --light
```

## Command-line reference

```
asciify <image> [options]
```

| Option | Default | Description |
|---|---|---|
| `<image>` | — | Input image path (required). PNG or JPEG. |
| `-o`, `--output <file>` | `ascii.png` | Output PNG path. |
| `--cols <n>`, `--width <n>` | `100` | Number of glyph columns. Must be ≥ 10. Row count is derived from it. |
| `--crop <x1,y1,x2,y2>` | — | Use only this pixel rectangle. Must satisfy `x2 > x1`, `y2 > y1` and stay inside the image. |
| `--gamma <f>` | `1.6` | Tone curve. `> 1` darkens, `< 1` brightens. |
| `--size <px>` | `14` | Font size in the output PNG. Larger = higher resolution and a larger file. |
| `--soft-bg` | off | Blur and fade the background when the image has no transparency. |
| `--light` | off | White background with dark glyphs (default is a dark background). |
| `-h`, `--help` | — | Print the usage text and exit. |

A status line is written to `stderr` on success:

```
Hazırdır: ascii.png (753x756 px, 100 sütun x 52 sətir)
             │              │          │            └── rows
             │              │          └── columns
             │              └── pixel size of the PNG
             └── output file
```

Exit codes: `0` on success, `1` on any error (the message goes to `stderr`).

## Examples

```bash
# default look
cargo run --release -- photo.png -o ascii.png

# crop to a face, softer background, wider grid
cargo run --release -- photo.png -o ascii.png \
  --crop 120,80,520,640 --cols 140 --gamma 1.6 --soft-bg

# light theme, high-resolution output
cargo run --release -- photo.png -o ascii.png --light --size 24

# brighten a dark photo
cargo run --release -- photo.png -o ascii.png --gamma 0.8
```

## How it works

The pipeline lives in `main.rs`; each step is handled by one module.

```
cli::parse_args          →  Opts
font::Font::load         →  font + glyph-cell metrics

image::load              →  ImageData   { gray, alpha, has_alpha, w, h }
  decode → crop → Rec.601 grayscale → alpha mask (0..1)

processing::process      →  ProcessedImage { values, w, h }
  1. local_contrast      CLAHE, 6x6 tiles, contrast limit 3.0
  2. background mask     alpha / --soft-bg / disabled
  3. stretch_contrast    2%..98% percentiles of the subject, then gamma
  4. sharpen             unsharp mask: 1.6·v − 0.6·blur(σ = 2.0)
  5. fade_background     push the background towards black or white

ascii::convert           →  AsciiImage { lines, cols, rows }
  resize to cols × rows (triangle filter) → map tone to a glyph

render::save             →  PNG on disk
  draw every glyph with ab_glyph (anti-aliased)
```

**Glyph mapping.** A tone of 0 is the darkest glyph and 255 the lightest. The ramp is

```
"@%#*+=-:. "     dense → sparse
```

On a dark background the ramp is reversed so that bright areas of the photo become
dense glyphs; on `--light` it is used as written.

**Row count.** A monospace glyph cell is taller than it is wide, so the number of
rows is derived from the image aspect ratio *and* the cell aspect ratio:

```
rows = round(cols × (image_height / image_width) × (cell_width / cell_height))
```

This keeps the PNG's proportions close to the original photo instead of stretching it.

**Output size.** `padding = round(--size)` pixels surround the grid:

```
width  = ceil(cols × cell_width)  + 2 × padding
height =     rows × cell_height   + 2 × padding
```

## Background handling

The behaviour depends on whether the input has transparency and on `--soft-bg`.
The first matching mode wins.

| Mode | When | What happens |
|---|---|---|
| **Alpha mask** | the image has any pixel with `alpha < 250` | The alpha channel, blurred by `0.004 × width`, becomes the mask. Transparent pixels are first filled with the mean tone of the subject so the contrast statistics are not skewed by empty space. Cleanest result. |
| **Soft background** | no alpha, and `--soft-bg` is passed | An ellipse centred at `(0.52, 0.50)` of the frame with radii `(0.45, 0.50)` is blurred by `0.068 × width` to produce the mask. Inside the ellipse the tone is kept; outside it is replaced by a `0.027 × width` blur, then faded out by `0.10 + 0.90 × mask`. |
| **Disabled** | neither of the above | No masking and no fading — the whole image is converted as it is. |

A face or portrait on a plain background benefits most from the first two modes.

## Project layout

Ten modules, one concern each. The largest is `mask.rs` at 150 lines.

| File | Lines | Responsibility |
|---|---|---|
| `src/main.rs` | 54 | Entry point; wires the pipeline stages together. |
| `src/cli.rs` | 125 | `Opts`, usage text, argument parsing and validation. |
| `src/image.rs` | 107 | Decode, crop, Rec.601 grayscale, alpha channel → `ImageData`. |
| `src/clahe.rs` | 90 | Contrast Limited Adaptive Histogram Equalization. |
| `src/ops.rs` | 27 | Shared buffer helpers: `to_gray_u8`, `blur`, `percentile`. |
| `src/mask.rs` | 150 | Background mask: alpha, `--soft-bg`, or disabled. |
| `src/processing.rs` | 105 | Contrast stretch, sharpening, background fade → `ProcessedImage`. |
| `src/font.rs` | 32 | Font loading and glyph-cell metrics, loaded exactly once. |
| `src/ascii.rs` | 75 | Pixels → glyphs → `AsciiImage`. |
| `src/render.rs` | 102 | Glyphs → PNG, with the dark/light palettes. |

## Design notes

- **CLAHE before the contrast stretch.** Local equalisation first, then a global
  2%–98% stretch, gives an even tone distribution without blowing out highlights.
- **The contrast range is measured inside the mask only.** Background pixels are
  excluded from the percentile calculation, so a dark or busy background cannot
  compress the subject's tonal range.
- **Gamma is inverted in `--light` mode.** On a dark theme bright means a dense
  glyph; on a light theme that mapping is reversed, otherwise the subject's light
  tones would collapse into a solid `"@"` blob.
- **The font is loaded once.** `ascii` (grid dimensions) and `render` (PNG size and
  drawing) share the same metrics, so the grid and the drawn image can never
  disagree.
- **`percentile` sorts with `f32::total_cmp`.** Identical ordering for finite values
  to `partial_cmp`, but it cannot panic on a NaN.
- **Tunable values are named constants.** Every blur radius, percentile, mask
  threshold, fade weight and sharpen coefficient is a named constant at the top of
  its module, so the look can be adjusted without hunting through the pipeline.

## Error messages

The CLI messages are in Azerbaijani. Reference:

| Message | Meaning |
|---|---|
| `Şəkil faylı verilməyib.` | No input image was given. |
| `Naməlum seçim: <flag>` | Unrecognised option, followed by the usage text. |
| `<option> üçün dəyər yoxdur` | The option at the end of the command has no value. |
| `--crop: <parse error>` | The crop argument is not four numbers. |
| `--crop formatı: x1,y1,x2,y2 (x2>x1, y2>y1)` | Crop is malformed or inverted. |
| `--crop şəkildən kənara çıxır (şəkil WxH)` | Crop extends past the image bounds. |
| `--cols ən azı 10 olmalıdır` | Column count is below the minimum of 10. |
| `--gamma ədəd olmalıdır` | Gamma is not a number (NaN). |
| `Hazırdır: <file> (…)` | Success — this is the normal completion message. |
