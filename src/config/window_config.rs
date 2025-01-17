use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use winit::dpi::LogicalSize;
use winit::window::WindowLevel;
use winit::window::WindowAttributes;

use crate::{config::scion_config::ScionConfig, graphics::components::color::Color};

/// Main configuration for the game window
/// Please use [`WindowConfigBuilder`] if you want to build if from code.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    /// Enables fullscreen mode
    pub(crate) fullscreen: bool,
    /// Default window width and height in pixels.
    pub(crate) dimensions: Option<(u32, u32)>,
    /// Minimum window width and height in pixels.
    pub(crate) min_dimensions: Option<(u32, u32)>,
    /// Maximum window width and height in pixels.
    pub(crate) max_dimensions: Option<(u32, u32)>,
    /// Whether to display the window, Use full for loading
    pub(crate) visibility: bool,
    /// The path relative to the game executable of the window icon.
    pub(crate) icon: Option<PathBuf>,
    /// Whether the window should always be on top of other windows.
    pub(crate) always_on_top: bool,
    /// Whether the window should have borders and bars.
    pub(crate) decorations: bool,
    /// Whether the window should be maximized upon creation.
    pub(crate) maximized: bool,
    /// If the user can resize the window
    pub(crate) resizable: bool,
    /// If the window should be able to be transparent.
    pub(crate) transparent: bool,
    /// Default background color of each frame in the window
    pub(crate) default_background_color: Option<Color>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            fullscreen: false,
            dimensions: Some((1024, 768)),
            min_dimensions: Some((384, 336)),
            max_dimensions: None,
            visibility: true,
            icon: None,
            always_on_top: false,
            decorations: true,
            maximized: false,
            resizable: true,
            transparent: false,
            default_background_color: None,
        }
    }
}

impl WindowConfig {
    pub(crate) fn into(self, scion_config: &ScionConfig) -> WindowAttributes {
        let mut builder = WindowAttributes::default();

        builder = builder.with_title(scion_config.app_name.clone())
            .with_fullscreen(None);
        if self.dimensions.is_some() {
            builder = builder.with_inner_size(self.dimensions.map(|d| LogicalSize::new(d.0, d.1)).unwrap())
        }
        if self.min_dimensions.is_some() {
            builder = builder.with_min_inner_size(self.min_dimensions.map(|d| LogicalSize::new(d.0, d.1)).unwrap())
        }
        if self.max_dimensions.is_some() {
            builder = builder.with_max_inner_size(self.max_dimensions.map(|d| LogicalSize::new(d.0, d.1)).unwrap())
        }

        builder
            .with_visible(self.visibility)
            .with_window_icon(None)
            .with_window_level(if self.always_on_top { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal })
            .with_decorations(self.decorations)
            .with_maximized(self.maximized)
            .with_resizable(self.resizable)
            .with_transparent(self.transparent)
    }
}

/// `WindowConfigBuilder` is a convenience builder to create a `WindowConfig` from code.
pub struct WindowConfigBuilder {
    config: WindowConfig,
}

impl Default for WindowConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowConfigBuilder {
    /// Create a new `WindowConfigBuilder` builder
    pub fn new() -> Self {
        Self { config: Default::default() }
    }

    /// Dimension of the window
    pub fn with_dimensions(mut self, dimensions: (u32, u32)) -> Self {
        self.config.dimensions = Some(dimensions);
        self
    }

    /// Whether or not the window should be resizable
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// When rendering to the window, which color to use as default
    pub fn with_default_background_color(mut self, color: Option<Color>) -> Self {
        self.config.default_background_color = color;
        self
    }

    /// Retrieves the configuration built
    pub fn get(self) -> WindowConfig {
        self.config
    }
}
