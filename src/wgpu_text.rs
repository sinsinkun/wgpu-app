#![allow(dead_code)]

use ab_glyph::{Font, FontRef, Glyph, Rect};
use image::{RgbaImage, Rgba};
use wgpu::{Extent3d, ImageCopyTexture, Queue, Texture, Origin3d, TextureAspect, ImageDataLayout};

#[derive(Debug, PartialEq)]
pub enum TextError {
  FileLoadError,
  GlyphOutlineError,
}

// create image of glyph to append onto 
pub fn load_new_glyph(c: char, color: [u8; 3]) -> Result<RgbaImage, TextError> {
  // open font
  let font = FontRef::try_from_slice(include_bytes!("../assets/roboto.ttf"))
    .map_err(|_| TextError::FileLoadError)?;

  // declare glyph
  let glyph: Glyph = font.glyph_id(c).with_scale(160.0);

  if let Some(ch) = font.outline_glyph(glyph) {
    // define image bounds
    let bounds: Rect = ch.px_bounds();
    let w = bounds.max.x - bounds.min.x;
    let h = bounds.max.y - bounds.min.y;
    // define image buffer
    let mut img = RgbaImage::new(w as u32, h as u32);

    // write pixels to image
    ch.draw(|x, y, c| {
      let r = color[0];
      let g = color[1];
      let b = color[2];
      let a: u8 = f32::floor(c * 255.0) as u8;
      img.put_pixel(x, y, Rgba([r,g,b,a]));
    });

    Ok(img)
  } else {
    Err(TextError::GlyphOutlineError)
  }
}

pub fn draw_glyph_texture(queue: &Queue, texture: &mut Texture, glyph: RgbaImage) {
  // define glyph data
  let dimensions = glyph.dimensions();
  let glyph_size = Extent3d { 
    width: dimensions.0,
    height: dimensions.1,
    depth_or_array_layers: 1
  };
  // write glyph to texture
  queue.write_texture(
    ImageCopyTexture {
      texture: &texture,
      mip_level: 0,
      origin: Origin3d { x:10, y:10, z:0 },
      aspect: TextureAspect::All,
    },
    &glyph,
    ImageDataLayout {
      offset: 0,
      bytes_per_row: Some(4 * dimensions.0),
      rows_per_image: Some(dimensions.1),
    },
    glyph_size
  );

}

#[cfg(test)]
mod glyph_brush_test {
  use super::*;

  #[test]
  fn glyph_test() {
    let _result = load_new_glyph('d', [100, 10, 100]);
    assert_eq!(1, 2);
  }

}