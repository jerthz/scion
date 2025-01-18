use std::collections::HashMap;
use std::path::Path;

use ab_glyph::{point, Font, FontVec, Glyph, Point, PxScale, ScaleFont};
use image::{DynamicImage, Rgba};
use log::info;
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Texture;
use crate::utils::file::{app_base_path, read_file};
use crate::utils::ScionError;

const TEXT: &str = "a b c d e f g h i j k l m n o p q r s t u v w x y z A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 1 2 3 4 5 6 7 8 9 0 é è à ù ç - ? ! . , : = / + - % & ' ( )";

#[derive(Debug)]
pub(crate) struct CharacterPosition {
    pub(crate) start_x: f32,
    pub(crate) start_y: f32,
    pub(crate) end_x: f32,
    pub(crate) end_y: f32,
}

impl CharacterPosition {
    pub fn new(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }

    pub fn width(&self) -> f32 {
        self.end_x - self.start_x
    }

    pub fn height(&self) -> f32 {
        self.end_y - self.start_y
    }
}

#[derive(Debug)]
pub(crate) struct FontAtlasEntry {
    pub(crate) texture: Option<Texture>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) min_y: f32,
    pub(crate) character_positions: HashMap<char, CharacterPosition>,
}

impl FontAtlasEntry {
    pub(crate) fn take_texture(&mut self) -> Texture {
        if let Some(tex) = self.texture.take(){
            return tex;
        }
        panic!("No texture");
    }
    pub(crate) fn compute_vertical_offset(&self, current_pos_y: f32) -> f32 {
        if current_pos_y > self.min_y {
            return current_pos_y - self.min_y
        }
        0.
    }
    pub(crate) fn min_y(&self) -> f32 {
        self.character_positions.iter()
            .min_by(|p1, p2| p1.1.start_y.partial_cmp(&p2.1.start_y).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| p.1.start_y).unwrap_or(0.)
    }
}

#[derive(Default)]
pub(crate) struct FontAtlas {
    atlas: HashMap<String, FontAtlasEntry>,
}

impl FontAtlas {
    pub fn get_texture_from_path(&mut self, path: &str) -> Option<&mut FontAtlasEntry> {
        if self.atlas.contains_key(path) {
            return self.atlas.get_mut(path);
        }
        None
    }

    pub fn get_texture(&self, font: &str, font_size: usize, font_color: &Color) -> Option<&FontAtlasEntry> {
        let key = FontAtlas::true_type_path(font, font_size, font_color);
        if self.atlas.contains_key(&key) {
            return self.atlas.get(&key);
        }
        None
    }

    pub fn true_type_path(font: &str, font_size: usize, font_color: &Color) -> String {
        format!("{:?}_{:?}_{:?}", font, font_size, font_color.to_string())
    }

    pub fn add_true_type(&mut self, font: String, font_size: usize, font_color: &Color, data: FontAtlasEntry) {
        let key = format!("{:?}_{:?}_{:?}", font, font_size, font_color.to_string());
        self.atlas.insert(key, data);
    }

    pub fn add_bitmap(&mut self, font: String, data: FontAtlasEntry) {
        self.atlas.insert(font, data);
    }
}

pub(crate) fn convert_true_type(font_path: String, font_size: usize, font_color: &Color) -> Result<FontAtlasEntry, ScionError> {
    match read_file(Path::new(&font_path)) {
        Ok(res) => {
            let font = FontVec::try_from_vec(res);
            if let Ok(font_vec) = font {
                let mut glyphs = Vec::<Glyph>::new();
                let scale = PxScale::from(font_size as f32);
                let scaled_font = font_vec.as_scaled(scale);
                layout_paragraph(scaled_font, point(20.0, 20.0), 9999.0, TEXT, &mut glyphs);
                let glyphs_height = scaled_font.height().ceil() as u32;
                let glyphs_width = {
                    let min_x = glyphs.first().unwrap().position.x;
                    let last_glyph = glyphs.last().unwrap();
                    let max_x = last_glyph.position.x + scaled_font.h_advance(last_glyph.id);
                    (max_x - min_x).ceil() as u32
                };

                let mut character_positions = HashMap::<char, CharacterPosition>::new();
                let mut min_y = 99999.;
                let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();
                let mut min_x = f32::MAX;
                let mut max_x = f32::MIN;
                for (pos, glyph) in glyphs.drain(0..glyphs.len()).enumerate() {
                    if let Some(outlined) = scaled_font.outline_glyph(glyph) {
                        let bounds = outlined.px_bounds();
                        outlined.draw(|x, y, v| {
                            let px = image.get_pixel_mut(x + bounds.min.x as u32, y + bounds.min.y as u32);
                            *px = Rgba([
                                font_color.red(),
                                font_color.green(),
                                font_color.blue(),
                                px.0[3].saturating_add((v * 255.0) as u8),
                            ]);
                        });
                        if min_y > bounds.min.y {
                            min_y = bounds.min.y;
                        }
                        let char_pos = CharacterPosition::new(bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y);
                        character_positions.insert(TEXT.to_string().chars().nth(pos).unwrap(), char_pos);
                    }
                }
                image.save(app_base_path().join("test_font.png").get()).unwrap();
                return Ok(FontAtlasEntry {
                    texture: Some(Texture {
                        bytes: image.to_vec(),
                        width: glyphs_width + 40,
                        height: glyphs_height + 40,
                    }),
                    width: glyphs_width + 40,
                    height: glyphs_height + 40,
                    min_y,
                    character_positions,
                });
            }
            Err(ScionError::new("Impossible to read font"))
        }
        Err(_) => Err(ScionError::new("Impossible to find font file"))
    }
}

pub(crate) fn convert_bitmap(texture_path: String,
                             chars: String,
                             width: f32,
                             height: f32,
                             texture_columns: f32,
                             texture_lines: f32) -> Result<FontAtlasEntry, ScionError> {

    match image::open(&texture_path) {
        Ok(img) => {
            let mut character_positions = HashMap::<char, CharacterPosition>::new();
            let img_width = img.width();
            let img_height = img.height();

            for (pos, character) in chars.chars().enumerate() {
                info!("c {}", texture_columns);
                let mut cursor_x = (pos % texture_columns as usize) as f32 * width;
                let mut cursor_y = (pos / texture_columns as usize) as f32 * height;
                let char_pos = CharacterPosition::new(cursor_x, cursor_y, cursor_x + width, cursor_y + height);
                character_positions.insert(character, char_pos);
            }

            Ok(FontAtlasEntry {
                texture: Some(Texture {
                    bytes: img.into_bytes(),
                    width: img_width,
                    height: img_height,
                }),
                width: img_width,
                height: img_height,
                min_y: 0.,
                character_positions,
            })
        }
        Err(err) => {
            Err(crate::utils::ScionError::new(""))
        }
    }
}

pub fn layout_paragraph<F, SF>(
    font: SF,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + max_width {
            caret = point(position.x, caret.y + v_advance);
            glyph.position = caret;
            last_glyph = None;
        }

        target.push(glyph);
    }
}