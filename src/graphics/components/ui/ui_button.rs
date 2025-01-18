use crate::core::components::maths::padding::Padding;
use crate::core::resources::asset_manager::AssetRef;
use crate::core::world::Resources;
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Material;
use crate::graphics::components::ui::font::Font;
use crate::graphics::components::ui::Focusable;

pub struct UiButton {
    text: String,
    width: usize,
    height: usize,
    background: Option<AssetRef<Material>>,
    hover: Option<AssetRef<Material>>,
    clicked: Option<AssetRef<Material>>,
    font_ref: AssetRef<Font>,
    /// font size when using a TrueType font
    font_size: usize,
    /// font color when using a TrueType font
    font_color: Option<Color>,
    tab_index: usize,
    padding: Padding,
    pub(crate) on_click: Option<fn(&mut Resources)>,
    pub(crate) hovered: bool,
    pub(crate) dirty: bool,
}

impl UiButton {
    pub fn new(width: usize, height: usize, font_ref: AssetRef<Font>) -> Self {
        Self {
            text: "".to_string(),
            width,
            height,
            background: None,
            hover: None,
            clicked: None,
            font_ref,
            font_size: 0,
            font_color: None,
            tab_index: 0,
            padding: Padding::default(),
            on_click: None,
            hovered: false,
            dirty: true
        }
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
    pub fn background(&self) -> Option<AssetRef<Material>> {
        self.background.clone()
    }
    pub fn clone_background_unchecked(&self) -> AssetRef<Material> {
        self.background.as_ref().expect("Unchecked unwrap of hover failed").clone()
    }
    pub fn hover(&self) -> Option<AssetRef<Material>> {
        self.hover.clone()
    }

    pub fn clone_hover_unchecked(&self) -> AssetRef<Material> {
        self.hover.as_ref().expect("Unchecked unwrap of hover failed").clone()
    }

    pub fn clicked(&self) -> Option<AssetRef<Material>> {
        self.clicked.clone()
    }

    pub fn clone_clicked_unchecked(&self) -> AssetRef<Material> {
        self.clicked.as_ref().expect("Unchecked unwrap of hover failed").clone()
    }
    pub fn font_size(&self) -> usize {
        self.font_size
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn padding(&self) -> Padding {
        self.padding
    }
    pub fn font_ref(&self) -> AssetRef<Font> {
        self.font_ref.clone()
    }


    pub fn with_font_size(mut self, font_size: usize) -> Self{
        self.font_size = font_size;
        self
    }

    pub fn with_font_color(mut self, color: Color) -> Self{
        self.font_color = Some(color);
        self
    }

    pub fn with_background_material(mut self, asset_ref: AssetRef<Material>) -> Self{
        self.background = Some(asset_ref);
        self
    }
    pub fn with_hover_material(mut self, asset_ref: AssetRef<Material>) -> Self{
        self.hover = Some(asset_ref);
        self
    }
    pub fn with_clicked_material(mut self, asset_ref: AssetRef<Material>) -> Self{
        self.clicked = Some(asset_ref);
        self
    }

    pub fn with_tab_index(mut self, tab_index: usize) -> Self{
        self.tab_index = tab_index;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self{
        self.padding = padding;
        self
    }

    pub fn with_on_click_action(mut self, on_click: fn(&mut Resources)) -> Self{
        self.on_click = Some(on_click);
        self
    }


    pub fn set_text(&mut self, text: String) {
        if text.ne(&self.text) {
            self.text = text;
            self.dirty = true;
        }
    }

    pub fn with_text(mut self, text: &str) -> Self{
        self.text = text.to_string();
        self
    }
}

impl Focusable for UiButton {
    fn tab_index(&self) -> usize {
        self.tab_index
    }

    fn set_tab_index(&mut self, tab_index: usize) {
        self.tab_index = tab_index;
    }
}