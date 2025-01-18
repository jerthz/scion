use std::fmt::{Display, Formatter};
use std::path::Path;

use image::{DynamicImage, GenericImage, ImageBuffer, ImageFormat};


use crate::{
    graphics::components::{color::Color, tiles::tileset::Tileset},
    utils::file::read_file,
};


/// Component used by the 2D Renderer to know which material to use when graphics a renderable object.
#[derive(Clone, Debug)]
pub enum Material {
    /// Fill with a color
    Diffuse(Color),
    /// Use a texture. Note that this means the target object will need to have uv maps.
    Texture(String),
    /// Tileset Texture. This will be added by the engine on entities with a sprite component.
    Tileset(Tileset),
}

impl Material {
    /// Returns the tile_size in case of a Tileset Material.
    pub(crate) fn tile_size(material: &Material) -> Option<usize> {
        if let Material::Tileset(tileset) = material {
            Some(tileset.tile_width) // FIXME
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct Texture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Texture {
    pub fn from_png(file_path: &Path) -> Texture {
        if let Ok(bytes) = read_file(file_path) {
            let converted_image = image::load_from_memory_with_format(&bytes, ImageFormat::Png);
            if let Ok(image) = converted_image {
                return Texture::create_texture_from_dynamic_image(image);
            }
        }
        log::error!("Error while loading your texture, loading fallback texture instead.");
        Texture::fallback_texture()
    }

    pub fn from_color(color: &Color) -> Texture {
        let img = ImageBuffer::from_fn(1, 1, |_x, _y| {
            image::Rgba([color.red(), color.green(), color.blue(), (color.alpha() * 255.) as u8])
        });
        Texture { bytes: img.into_raw(), width: 1, height: 1 }
    }

    fn fallback_texture() -> Texture {
        let bytes = include_bytes!("missing_texture.png");
        let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
            .expect("The fallback image has not been found !");
        Texture::create_texture_from_dynamic_image(image)
    }

    fn create_texture_from_dynamic_image(dynamic_image: DynamicImage) -> Texture {
        let image = dynamic_image.to_rgba8();
        let width = image.width();
        let height = image.height();
        let bytes = image.into_raw();

        Texture { bytes, width, height }
    }
}

#[derive(Debug)]
pub(crate) struct TextureArray {
    pub(crate) bytes_array: Vec<Vec<u8>>,
    pub(crate) unit_width: u32,
    pub(crate) unit_height: u32,
    pub(crate) lines: u32,
}

impl TextureArray {
    pub fn from_tileset(tileset: &Tileset) -> Self {
        if let Ok(bytes) = read_file(Path::new(tileset.texture.as_str())) {
            let converted_image = image::load_from_memory_with_format(&bytes, ImageFormat::Png);
            if let Ok(image) = converted_image {
                return Self::create_texture_array_from_dynamic_image(image, tileset.height as u32, tileset.width as u32, tileset.tile_width as u32, tileset.tile_height as u32);
            }
        }
        log::error!("Error while loading your texture, loading fallback texture instead.");
        TextureArray::fallback_texture_array(tileset.height, tileset.width, tileset.tile_width as u32, tileset.tile_height as u32)
    }

    fn create_texture_array_from_dynamic_image(mut dynamic_image: DynamicImage,
                                               nb_lines: u32,
                                               nb_columns: u32,
                                               split_width: u32,
                                               split_height: u32) -> Self {
        let mut array: Vec<Vec<u8>> = Vec::new();

        for x in 0..nb_lines {
            for y in 0..nb_columns {
                let sub_image = dynamic_image.sub_image(y * split_width, x * split_height, split_width, split_height);
                array.push(sub_image.to_image().into_raw());
            }
        }

        Self {
            bytes_array: array,
            unit_width: split_width,
            unit_height: split_height,
            lines: nb_lines
        }
    }

    fn fallback_texture_array(nb_lines: usize, nb_columns: usize, split_width: u32, split_height: u32) -> TextureArray {
        let mut array: Vec<Vec<u8>> = Vec::new();
        for _x in 0..nb_columns {
            for _y in 0..nb_lines {
                array.push(Texture::fallback_texture().bytes);
            }
        }
        Self {
            bytes_array: array,
            unit_width: split_width,
            unit_height: split_height,
            lines: nb_lines as u32
        }
    }
}

impl Display for TextureArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "array length : {}, unit width : {}, unit height : {}", self.bytes_array.len(), self.unit_width, self.unit_height)
    }
}
