/// [`Font`] represents the different fonts available in `Scion`
#[derive(Clone)]
pub enum Font {
    /// Texture based font
    Bitmap {
        /// Path to the texture of this font, PNG only.
        texture_path: String,
        /// List of characters available in the font, in the right order
        chars: String,
        /// Character width in pixel
        width: f32,
        /// Character height in pixel
        height: f32,
        /// Number of column in the font's texture
        texture_columns: f32,
        /// Number of lines in the font's texture
        texture_lines: f32,
    },
    TrueType {
        font_path: String
    }
}

