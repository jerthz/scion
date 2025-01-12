pub mod font;
pub mod ui_image;
pub mod ui_text;
pub mod ui_input;
pub mod ui_button;

/// A component added by systems to each ui component's entity.
pub(crate) struct UiComponent;

/// A component added by systems to each focusable ui component's entity.
pub(crate) struct UiFocusable{
    pub(crate) rank: usize,
    pub(crate) focused: bool
}

/// trait that should be shared by all focusable components
pub trait Focusable {
    fn tab_index(&self) -> usize;
    fn set_tab_index(&mut self, tab_index: usize);
}
