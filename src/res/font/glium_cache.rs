use glium;
use rusttype::{self, PositionedGlyph, FontCollection, Font};
use std;
use std::collections::BTreeMap;
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use res::font::{FontCache, GlyphLookup, CacheGlyphError, CacheReadError, FontSpec, FontHandle};

pub struct GliumGlyphLookup<'a> {
  /// A map of font handles to actual font objects, with an associated x and y
  /// scale.
  fonts: BTreeMap<FontHandle, (Font<'a>, (f32, f32))>,
  /// The cache (not including actual texture storage).
  cache: rusttype::gpu_cache::Cache,
}

/// An implementation of a font cache using glium to cache the glyph textures
/// in vRAM.
pub struct GliumFontCache<'a> {
  /// A map of font specs to handles. If a font spec is loaded again, it will
  /// be stored under the same font handle as before.
  font_handles: BTreeMap<FontSpec, FontHandle>,
  /// A counter for the next font handle. This will always store the value of
  /// the next available font handle.
  curr_font_handle: FontHandle,
  /// A struct which can be handed out to multiple threads to lookup the UVs of glyphs.
  glyph_lookup: Arc<GliumGlyphLookup<'a>>,
  /// The texture storage for the font cache.
  cache_tex: glium::texture::srgb_texture2d::SrgbTexture2d,
}
impl<'a> std::fmt::Debug for GliumFontCache<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    write!(f, r#"GliumFontCache {{ font_handles: BTreeMap, 
           glyphs: BTreeMap, curr_font_handle: {:?}, 
           cache: rusttype::gpu_cache::Cache, cache_tex: Texture2d }}"#, 
           self.curr_font_handle)
  }
}

impl<'a> GliumFontCache<'a> {
  pub fn new<F: glium::backend::Facade>(display: &F) -> GliumFontCache<'a> {
    const CACHE_W : u32 = 4096;
    const CACHE_H : u32 = 4096;
    GliumFontCache {
      font_handles: BTreeMap::new(),
      curr_font_handle: FontHandle(0),
      // 2048 * 2048 cache with 0.1 scale tolerance and 1.0 position fault
      // tolerance (we aren't using positioning).
      glyph_lookup: Arc::new(GliumGlyphLookup {
        fonts: BTreeMap::new(),
        cache: rusttype::gpu_cache::Cache::new(CACHE_W, CACHE_H, 0.1, 1.0),
      }),
      // Create a new glium 2d texture with the cache width and height as the texture size.
      cache_tex: glium::texture::srgb_texture2d::SrgbTexture2d::with_format(
        display,
        glium::texture::RawImage2d {
          data: Cow::Owned(vec![0u8; CACHE_W as usize * CACHE_H as usize]),
          width: CACHE_W,
          height: CACHE_H,
          format: glium::texture::ClientFormat::U8
        },
        glium::texture::SrgbFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap).unwrap(),
    }
  }

  pub fn get_glyph_lookup(&'a self) -> Arc<GliumGlyphLookup<'a>> {
      self.glyph_lookup.clone()
  }

  /// Gets the next unique, unused font handle
  fn get_next_font_handle(&mut self) -> FontHandle {
    let fh = self.curr_font_handle;
    self.curr_font_handle.0 += 1;
    return fh;
  }

  pub fn get_tex(&self) -> &glium::texture::srgb_texture2d::SrgbTexture2d { &self.cache_tex }
}

impl<'a> FontCache for GliumFontCache<'a> {
  fn cache_glyphs<F: AsRef<Path>>(&mut self, filepath: F, scale: f32, 
                                  charset: &[char]) -> Result<FontHandle, CacheGlyphError> {
    use std::fs::File;
    use std::io::Read;

    // Open the font file and read it all.
    let mut f = try!(File::open(filepath.as_ref()));
    let mut data = Vec::new();
    try!(f.read_to_end(&mut data));

    // Create a font from the font file bytes.
    let font = try!(FontCollection::from_bytes(data).into_font()
                    .ok_or(std::io::Error::new(
                        std::io::ErrorKind::InvalidData, 
                        "Font file did not contain a valid font.")));

    // See if there's a font handle already used by this font spec - If not,
    // create a new one and store it in the map.
    let fs = FontSpec::new(filepath, (scale*100.0) as u32, (scale*100.0) as u32);
    let fh : FontHandle;
    if self.font_handles.contains_key(&fs) {
      fh = *self.font_handles.get(&fs).unwrap();
    }
    else { 
      fh = self.get_next_font_handle(); 
      self.font_handles.insert(fs, fh);
    }

    // Check if these characters exist in the cache - if not, queue them for
    // caching.  First, linear search n times through charset to make sure
    // there are no duplicates.
    let mut no_dup = Vec::with_capacity(charset.len());
    for ii in 0..charset.len() {
      let mut dup = false;
      for jj in 0..charset.len() {
        if ii != jj && charset[ii] == charset[jj] {
          dup = true;
          break;
        }
      }
      if !dup {
        no_dup.push(charset[ii]);
      }
    }

    let glyph_lookup = Arc::get_mut(&mut self.glyph_lookup)
    .expect("Failed to acquire mutable reference when caching glyphs. Is the font cache in
            use?");

    // Clear the queue to make sure we don't cache glyphs we didn't explicitly
    // ask for in this function.
    glyph_lookup.cache.clear_queue();

    // Now run through the no_dup vec and try to call rect_for on the cache. If
    // an error is returned (for no rect found) then we can queue this glyph.
    let mut glyphs_not_found = Vec::new(); // The list of glyphs not found in this font
    for c in &no_dup {
      // Create the positioned glyph
      let plain_glyph = font.glyph(*c).unwrap();
      if plain_glyph.id().0 == 0 {
        glyphs_not_found.push(*c);
        continue;
      }
      let g = plain_glyph.standalone()
        .scaled(rusttype::Scale::uniform(scale))
        .positioned(rusttype::Point{x: 0.0, y: 0.0});

      // Look up the rect in the cache
      let res = glyph_lookup.cache.rect_for(fh.0, &g);
      let mut cached = true;
      match res {
        Err(rusttype::gpu_cache::CacheReadErr::GlyphNotCached) => cached = false,
        _ => ()
      }
      // If the glyph isn't cached, then queue the glyph
      if !cached {
        glyph_lookup.cache.queue_glyph(fh.0, g.clone());
      }
    }
    if glyphs_not_found.len() != 0 {
      glyph_lookup.cache.clear_queue();
      return Err(CacheGlyphError::GlyphNotSupported(glyphs_not_found));
    }

    let cache_tex = &mut self.cache_tex;
    // Cache the whole queue of glyphs
    try!(glyph_lookup.cache.cache_queued(move |rect, data| {
      cache_tex.main_level().write(glium::Rect {
        left: rect.min.x,
        bottom: rect.min.y,
        width: rect.width(),
        height: rect.height()
      }, glium::texture::RawImage2d {
        data: Cow::Borrowed(data),
        width: rect.width(),
        height: rect.height(),
        format: glium::texture::ClientFormat::U8
      });
    }).map_err(|_| CacheGlyphError::CacheTooSmall));

    if !glyph_lookup.fonts.contains_key(&fh) {
      glyph_lookup.fonts.insert(fh, (font, (scale, scale)));
    }

    return Ok(fh);
  }
}

impl<'a> GlyphLookup for GliumFontCache<'a> {
  fn rect_for(&self, font_handle: FontHandle, 
              code_point: char) -> Result<Option<[f32; 4]>, CacheReadError> {
    self.glyph_lookup.rect_for(font_handle, code_point)
  }

  fn get_font_ref(&self, fh: FontHandle) -> Option<&(Font, (f32, f32))> { 
      self.glyph_lookup.fonts.get(&fh) 
  }

  fn get_glyph(&self, fh: FontHandle, c: char) -> Option<PositionedGlyph> {
      self.glyph_lookup.get_glyph(fh, c)
  }
}

impl<'a> GlyphLookup for Arc<GliumGlyphLookup<'a>> {
  fn rect_for(&self, font_handle: FontHandle, 
              code_point: char) -> Result<Option<[f32; 4]>, CacheReadError> {
    let g = self.get_glyph(font_handle, code_point); // Get the glyph
    let g = try!(g.ok_or(CacheReadError));

    // Try and get the rect.     
    let rect_opt = try!(self.cache.rect_for(font_handle.0, &g));
    if rect_opt.is_none() { return Ok(None); }

    // UV rect and glyph screen pos rect
    let (uv_rect, _) = rect_opt.unwrap();
    Ok(Some([uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y]))
  }

  fn get_font_ref(&self, fh: FontHandle) -> Option<&(Font, (f32, f32))> { 
      self.fonts.get(&fh) 
  }

  fn get_glyph(&self, fh: FontHandle, c: char) -> Option<PositionedGlyph> {
    let f_x_y = self.fonts.get(&fh);
    if f_x_y.is_none() { return None; }
    let &(ref font, (x_scale, y_scale)) = f_x_y.unwrap();
    let plain_glyph = font.glyph(c).unwrap();
    if plain_glyph.id().0 == 0 { return None; }
    let g = plain_glyph.standalone()
      .scaled(rusttype::Scale{ x: x_scale, y: y_scale })
      .positioned(rusttype::Point{x: 0.0, y: 0.0});
    return Some(g);
  }
}
