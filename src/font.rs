use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont};

const FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSansMono.ttf");

/// Glyph used to measure the advance width (monospace, so every glyph matches).
const REFERENCE_GLYPH: char = 'M';

/// A font plus the metrics of a single glyph cell.
///
/// Both the glyph grid size (`ascii`) and the PNG size and drawing (`render`) must
/// be based on the same metrics, so the font is loaded only once.
pub struct Font {
    pub font: FontRef<'static>,
    pub scale: PxScale,
    pub cell_width: f32,
    pub cell_height: f32,
}

impl Font {
    pub fn load(size_px: f32) -> Result<Self, ab_glyph::InvalidFont> {
        let font = FontRef::try_from_slice(FONT_BYTES)?;
        let scale = PxScale::from(size_px);
        let scaled = font.as_scaled(scale);

        Ok(Self {
            cell_width: scaled.h_advance(font.glyph_id(REFERENCE_GLYPH)),
            cell_height: (scaled.height() + scaled.line_gap()).ceil(),
            font,
            scale,
        })
    }
}
