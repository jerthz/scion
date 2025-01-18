use crate::core::resources::asset_manager::AssetRef;
use crate::graphics::components::color::Color;
use crate::graphics::components::ui::font::Font;
use crate::graphics::components::ui::Focusable;

/// A component representing an input Text in the UI.
pub struct UiInput {
    text: String,
    width: usize,
    height: usize,
    _cursor: usize,
    font_ref: AssetRef<Font>,
    /// font size when using a TrueType font
    font_size: usize,
    /// font color when using a TrueType font
    font_color: Option<Color>,
    tab_index: usize,
    pub(crate) dirty: bool
}

impl UiInput{
    pub fn new(width: usize, height: usize, font_ref: AssetRef<Font>) -> Self{
        Self{
            text: "".to_string(),
            width,
            height,
            _cursor: 0,
            font_ref,
            font_size: 0,
            font_color: None,
            tab_index: 0,
            dirty: true,
        }
    }

    pub fn with_font_size(mut self, font_size: usize) -> Self{
        self.font_size = font_size;
        self
    }

    pub fn with_font_color(mut self, color: Color) -> Self{
        self.font_color = Some(color);
        self
    }

    pub fn with_tab_index(mut self, tab_index: usize) -> Self{
        self.tab_index = tab_index;
        self
    }


    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
    pub fn font_color(&self) -> Option<Color> {
        self.font_color.clone()
    }
    pub fn font_size(&self) -> usize {
        self.font_size
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn font_ref(&self) -> AssetRef<Font> {
        self.font_ref.clone()
    }

    /// sets the content of this `UiText`
    pub fn set_text(&mut self, text: String) {
        if text.ne(&self.text) {
            self.text = text;
            self.dirty = true;
        }
    }
}

impl Focusable for UiInput{
    fn tab_index(&self) -> usize {
        self.tab_index
    }

    fn set_tab_index(&mut self, tab_index: usize) {
        self.tab_index = tab_index;
    }
}


