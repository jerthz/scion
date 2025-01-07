use log::info;
use winit::window::CursorIcon;

#[derive(Debug, Copy, Clone)]
struct FutureSettings{
    new_cursor: Option<CursorIcon>,
    dimensions: Option<(u32, u32)>
}

impl Default for FutureSettings{
    fn default() -> Self {
        Self{
            new_cursor: None,
            dimensions: None,
        }
    }
}

/// [`Window`] is a Resource dedicated to have an access control over the current window.
/// Its size is immediately updated when the window resize event happens.
/// new_cursor is set at the end of the current frame.
#[derive(Default, Debug, Copy, Clone)]
pub struct Window {
    width: u32,
    height: u32,
    dpi: f64,
    current_cursor: Option<CursorIcon>,
    future_settings: FutureSettings
}

impl Window {
    pub(crate) fn new(screen_size: (u32, u32), dpi: f64) -> Self {
        Self { width: screen_size.0, height: screen_size.1, future_settings: Default::default(), dpi, current_cursor: None }
    }

    pub(crate) fn set_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_cursor(&mut self, icon: CursorIcon) {
        if let Some(current) = self.current_cursor.as_ref() {
            if current != &icon {
                self.future_settings.new_cursor = Some(icon);
            }
        }else{
            self.future_settings.new_cursor = Some(icon);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.future_settings.dimensions = Some((width, height));
    }

    pub fn set_dpi(&mut self, dpi: f64) {
        self.dpi = dpi;
    }

    pub(crate) fn reset_future_settings(&mut self) {
        info!("resetting cursor");
        self.current_cursor = self.future_settings.new_cursor;
        self.future_settings = FutureSettings::default();
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn width(&self) -> u32 {
        (self.width as f64 / self.dpi) as u32
    }

    pub fn height(&self) -> u32 {
        (self.height as f64 / self.dpi) as u32
    }

    pub fn dpi(&self) -> f64 {
        self.dpi
    }

    pub fn new_cursor(&self) -> &Option<CursorIcon> {
        &self.future_settings.new_cursor
    }
    pub fn new_dimensions(&self) -> &Option<(u32, u32)> {
        &self.future_settings.dimensions
    }
}
