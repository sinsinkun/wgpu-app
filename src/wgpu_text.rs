#![allow(dead_code)]

use ab_glyph::{Font, FontRef, Glyph, Rect};
use image::{RgbaImage, Rgba};
use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect, TextureFormat};

#[derive(Debug, PartialEq)]
pub enum TextError {
  FileNotFound,
  FileLoadError,
  GlyphOutlineError,
  ExceedsBounds,
}

pub struct RStringInputs<'a> {
  pub queue: &'a Queue,
  pub texture: &'a mut Texture,
  pub font_data: &'a Vec<u8>,
  pub string: &'a str,
  pub size: f32,
  pub color: [u8; 3],
  pub top_left: [u32; 2],
  pub char_gap: u32,
}

// create image of glyph to append onto texture
pub fn load_new_glyph(c: char, color: [u8; 3]) -> Result<RgbaImage, TextError> {
  // open font
  // let font = FontRef::try_from_slice(include_bytes!("../assets/retro_computer.ttf"))
  //   .map_err(|_| TextError::FileLoadError)?;
  let font = FontRef::try_from_slice(include_bytes!("embed_assets/roboto.ttf"))
    .map_err(|_| TextError::FileLoadError)?;

  // declare glyph
  let glyph: Glyph = font.glyph_id(c).with_scale(20.0);

  if let Some(ch) = font.outline_glyph(glyph) {
    // define image bounds
    let bounds: Rect = ch.px_bounds();
    println!("bounds? {bounds:?}");
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

// same as load_new_glyph but with cached font data
pub fn load_cached_glyph(font_raw: &Vec<u8>, c: char, size: f32, color: [u8; 3]) -> Result<RgbaImage, TextError> {
  let font = FontRef::try_from_slice(font_raw).map_err(|_| TextError::FileLoadError)?;
  let glyph: Glyph = font.glyph_id(c).with_scale(size);

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

// draw glyph on texture
pub fn draw_glyph_on_texture(queue: &Queue, texture: &mut Texture, glyph: RgbaImage, position: [u32; 2]) -> Result<(), TextError> {
  // define glyph data
  let dimensions = glyph.dimensions();
  let glyph_size = Extent3d { 
    width: dimensions.0,
    height: dimensions.1,
    depth_or_array_layers: 1
  };

  // early exit if not enough space on texture to render text
  let container_w = texture.width();
  let container_h = texture.height();
  if position[0] + dimensions.0 > container_w {
    return Err(TextError::ExceedsBounds)
  }
  if position[1] + dimensions.1 > container_h {
    return Err(TextError::ExceedsBounds)
  }

  // write glyph to texture
  queue.write_texture(
    ImageCopyTexture {
      texture: &texture,
      mip_level: 0,
      origin: Origin3d { x:position[0], y:position[1], z:0 },
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

  Ok(())
}

// combines glyph functions to render full string
pub fn draw_str(input: RStringInputs) -> Result<(), TextError> {
  // create individual glyph rasters
  let mut offset: u32 = 0;
  let mut glyphs: Vec<(u32, u32, u32, char, RgbaImage)> = Vec::new();
  let mut max_h: u32 = 0;

  // handle texture format conversion
  let t_fmt = input.texture.format();
  let mut color = input.color;
  match t_fmt {
    TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => {
      let b = color[0];
      color[0] = color[2];
      color[2] = b;
    }
    _ => ()
  }

  // convert characters to rasterized images
  for c in input.string.chars() {
    // skip empty characters (todo: handle newlines separately)
    if c == ' ' || c == '\n' || c == '\t' {
      offset += input.char_gap * 3;
      continue;
    }
    let glyph = load_cached_glyph(input.font_data, c, input.size, color)?;
    let x = input.top_left[0] + offset;
    let y = input.top_left[1];
    let h = glyph.height();
    
    if h > max_h { max_h = h }
    offset += glyph.width() + input.char_gap;
    glyphs.push((x, y, h, c, glyph));
  }

  // update glyph positions for better alignment
  for g in &mut glyphs {
    match g.3 {
      'g'|'j'|'q'|'p'|'y'|'+' => {
        g.1 += max_h / 4;
      }
      '-'|'=' => {
        g.1 += max_h / 2;
      }
      '\''|'\"'|'*'|'^' => (),
      _ => {
        if g.2 < max_h {
          let dh = max_h - g.2;
          g.1 += dh;
        }
      }
    }
  }

  // draw to texture
  for (x, y, _h, _c, img) in glyphs {
    draw_glyph_on_texture(input.queue, input.texture, img, [x, y])?;
  }

  Ok(())
}

#[cfg(test)]
mod glyph_brush_test {
  use super::*;

  #[test]
  fn glyph_test() {
    let _ = load_new_glyph('d', [100, 10, 100]);
    let _ = load_new_glyph('a', [100, 10, 100]);
    let _ = load_new_glyph('-', [100, 10, 100]);
    assert_eq!(1, 2);
  }

}