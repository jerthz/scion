use std::ops::Range;

use wgpu::{PrimitiveTopology, util::BufferInitDescriptor};

use crate::{
    graphics::components::{
        material::Material,
        ui::{font::Font, ui_image::UiImage},
    },
    graphics::rendering::{Renderable2D, RenderableUi},
};
use crate::graphics::components::color::Color;
use crate::core::components::maths::padding::Padding;
use crate::core::resources::asset_manager::AssetRef;
use crate::core::world::Resources;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertex;

/// A component representing a Text in the UI.
pub struct UiText {
    text: String,
    font_ref: AssetRef<Font>,
    /// font size when using a TrueType font
    font_size: usize,
    /// font color when using a TrueType font
    font_color: Option<Color>,
    /// Optional text settings when used in buttons
    padding: Padding,
    pub(crate) dirty: bool,
    pub(crate) sync_fn: Option<fn(&mut Resources) -> String>
}

impl UiText {
    /// Creates a new `UiText` with `text` as default content and `font`
    pub fn new(text: String, font_ref: AssetRef<Font>) -> Self {
        Self { text, font_ref, dirty: true, font_size: 10, font_color: None, sync_fn: None, padding: Padding::default() }
    }

    /// provide a fn that will automatically synchronize the text
    /// with the given value
    pub fn sync_value(mut self, sync_function: fn(&mut Resources) -> String) -> Self
    {
        self.sync_fn = Some(sync_function);
        self
    }

    /// retrieves the content of this `UiText`
    pub fn text(&self) -> &String {
        &self.text
    }

    /// retrieves the font size of this `UiText`. Font size is only used on TrueType fonts
    pub fn font_size(&self) -> usize {
        self.font_size
    }
    pub fn padding(&self) -> &Padding {
        &self.padding
    }

    /// retrieves the font color of this `UiText`. Font color is only used on TrueType fonts
    pub fn font_color(&self) -> &Option<Color> {
        &self.font_color
    }

    /// sets the content of this `UiText`
    pub fn set_text(&mut self, text: String) {
        if text.ne(&self.text) {
            self.text = text;
            self.dirty = true;
        }
    }

    pub fn set_padding(&mut self, padding: Padding) {
        self.padding = padding;
    }

    /// retrieve the font of this `UiText`
    pub fn font_ref(&self) -> &AssetRef<Font> {
        &self.font_ref
    }

    pub fn with_font_size(mut self, font_size: usize) -> Self{
        self.font_size = font_size;
        self
    }

    pub fn with_font_color(mut self, color: Color) -> Self{
        self.font_color = Some(color);
        self
    }


}

/// `UiTextImage` is an internal component used to keep track of the character in case of a
/// bitmap font
#[derive(Debug)]
pub(crate) struct UiTextImage(pub(crate)
                              UiImage);

impl Renderable2D for UiTextImage {
    fn vertex_buffer_descriptor(&mut self, material: Option<&Material>) -> BufferInitDescriptor {
        self.0.vertex_buffer_descriptor(material)
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        self.0.indexes_buffer_descriptor()
    }

    fn range(&self) -> Range<u32> {
        self.0.range()
    }

    fn topology() -> PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleList
    }

    fn dirty(&self) -> bool {
        false
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn get_rendering_priority(&self) -> usize {
        1
    }
}

impl RenderableUi for UiTextImage {}
