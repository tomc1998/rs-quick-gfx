use renderer::{Vertex, TexType};
use std;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
use res::font::glium_cache::GliumFontCache;
use res::font::{FontHandle, FontCache, CacheReadError};
use res::tex::{TexHandle};
use res::tex::glium_cache::GliumTexCache;
use vec::Vec2;
use rusttype::Scale;

#[derive(Copy, Clone, Hash, Debug)]
pub struct RenderTextError;
impl std::fmt::Display for RenderTextError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    use std::error::Error;
    write!(f, "{}", self.description())
  }
}
impl std::error::Error for RenderTextError {
  fn description(&self) -> &'static str { "Text rendering failed - not all glyphs were cached." }
}
impl std::convert::From<CacheReadError> for RenderTextError {
  fn from(_: CacheReadError) -> Self { RenderTextError }
}

#[derive(Copy, Clone, Hash, Debug)]
pub struct RenderTextureError;
impl std::fmt::Display for RenderTextureError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    use std::error::Error;
    write!(f, "{}", self.description())
  }
}
impl std::error::Error for RenderTextureError {
  fn description(&self) -> &'static str { "Texture rendering failed - texture wasn't cached." }
}
impl std::convert::From<CacheReadError> for RenderTextureError {
  fn from(_: CacheReadError) -> Self { RenderTextureError }
}


/// This struct wraps a Sender<Vec<Vertex>>, and has convenience methods to
/// draw certain geometry.
#[derive(Clone)]
pub struct RendererController<'a> {
  font_cache: Arc<Mutex<GliumFontCache<'a>>>,
  tex_cache: Arc<Mutex<GliumTexCache>>,
  white: TexHandle,
  sender: mpsc::Sender<Vec<Vertex>>,
}

impl<'a> RendererController<'a> {
  /// Creates a new renderer controller with a given mpsc sender. If you want
  /// to get a renderer controller, look at the
  /// renderer::Renderer::get_renderer_controller() function.
  pub fn new(sender: mpsc::Sender<Vec<Vertex>>, 
             font_cache: Arc<Mutex<GliumFontCache<'a>>>,
             tex_cache: Arc<Mutex<GliumTexCache>>,
             white: TexHandle) -> Box<RendererController<'a>> {
    Box::new(RendererController { 
      sender: sender, 
      font_cache: font_cache, 
      tex_cache: tex_cache,
      white: white,
    })
  }

  /// Lookup a texture handle, and transform the rectangle coordinates into x0,
  /// y0, x1, y1 (as opposed to x,y,w,h).
  fn lookup_tex(&self, tex: TexHandle) -> Option<(usize, [f32; 4])> {
    use res::tex::TexCache;
    // Get the index of this texture.
    let ix_rect_opt = { self.tex_cache.lock().unwrap().rect_for(tex) };
    if ix_rect_opt.is_none() { return None; }
    let (tex_ix, mut rect) = ix_rect_opt.unwrap();
    // Transform from x,y,w,h to x0,y0,x1,y1
    rect[2] = rect[0] + rect[2];
    rect[3] = rect[1] + rect[3];
    Some((tex_ix, rect))
  }

  /// Draws a line given a start and an endpoint.
  /// #Params
  /// * `p1` - The starting point
  /// * `p2` - The ending point
  /// * `w` - The line width
  /// * `col` - The colour of the line
  pub fn line(&self, p1: [f32; 2], p2: [f32; 2], w: f32, col: [f32; 4]) {
    let mut data = Vec::with_capacity(6);
    let p1 = Vec2(p1);
    let p2 = Vec2(p2);
    let half_w = w/2.0;
    let p1p2 = p2.sub(p1);

    // Get the 4 corners of the 'rectangle' (the line is just a rectangle)
    let perp_l_1 = Vec2([-p1p2[1], p1p2[0]]).nor().mul(half_w).add(p1);
    let perp_r_1 = Vec2([p1p2[1], -p1p2[0]]).nor().mul(half_w).add(p1);
    let perp_l_2 = Vec2([-p1p2[1], p1p2[0]]).nor().mul(half_w).add(p2);
    let perp_r_2 = Vec2([p1p2[1], -p1p2[0]]).nor().mul(half_w).add(p2);

    // Generate the vertex data
    // tri 1
    data.push(Vertex{ 
      pos: [perp_l_1[0], perp_l_1[1]], 
      col: col.clone(), 
      tex_coords: [0.0, 0.0], 
      tex_type: TexType::Texture, tex_ix: 0});
    data.push(Vertex{ 
      pos: [perp_r_1[0], perp_r_1[1]], 
      col: col.clone(), 
      tex_coords: [0.0, 0.0],
      tex_type: TexType::Texture, tex_ix: 0});
    data.push(Vertex{ 
      pos: [perp_l_2[0], perp_l_2[1]], 
      col: col.clone(), 
      tex_coords: [0.0, 0.0],
      tex_type: TexType::Texture, tex_ix: 0});

    // tri 2
    data.push(Vertex{ 
      pos: [perp_l_2[0], perp_l_2[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: 0,
      tex_coords: [0.0, 0.0]});
    data.push(Vertex{ 
      pos: [perp_r_2[0], perp_r_2[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: 0,
      tex_coords: [0.0, 0.0]});
    data.push(Vertex{ 
      pos: [perp_r_1[0], perp_r_1[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: 0,
      tex_coords: [0.0, 0.0]});

    // Send the vertex data through the sender
    self.sender.send(data).unwrap();
  }

  /// Draws a line given a start and an endpoint.
  /// #Params
  /// * `aabb` - The AABB box for the rectangle - X, Y, W, H
  /// * `col` - The colour of the rectangle
  pub fn rect(&self, aabb: &[f32; 4], col: &[f32; 4]) {
    let mut data = Vec::with_capacity(6);

    // Lookup white texture
    let (tex_ix, rect) = self.lookup_tex(self.white).unwrap();
    let t_x = (rect[0] + rect[2]) / 2.0;
    let t_y = (rect[1] + rect[3]) / 2.0;

    // Generate vertex data
    // Tri 1
    data.push( Vertex { 
      pos: [aabb[0], aabb[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });
    data.push( Vertex { 
      pos: [aabb[0] + aabb[2], aabb[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });
    data.push( Vertex { 
      pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });

    // Tri 2
    data.push( Vertex { 
      pos: [aabb[0], aabb[1]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });
    data.push( Vertex { 
      pos: [aabb[0], aabb[1] + aabb[3]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });
    data.push( Vertex { 
      pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], 
      col: col.clone(), 
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [t_x, t_y] });

    // Send the data
    self.sender.send(data).unwrap();
  }

  /// Draws a circle.
  /// # Params
  /// * `pos` The position on screen of the circle
  /// * `rad` The radius of the circle
  /// * `segments` The number of triangle segments to use when drawing. More = smoother circle.
  /// * `col` - The colour of the circle.
  pub fn circle(&self, pos: &[f32; 2], rad: f32, segments: usize, col: &[f32; 4]) {
    use std::f64::consts::PI;

    // Lookup white texture
    let (tex_ix, rect) = self.lookup_tex(self.white).unwrap();
    let t_x = (rect[0] + rect[2]) / 2.0;
    let t_y = (rect[1] + rect[3]) / 2.0;

    let mut data = Vec::with_capacity(segments*3);
    let mut curr_angle = 0.0f32;
    let angle_increment = 2.0*(PI as f32)*(1.0 / segments as f32);
    for _ in 0..segments {
      // Vertex at the centre of the circle
      data.push(Vertex {
        pos: pos.clone(), 
        col: col.clone(), 
        tex_type: TexType::Texture, tex_ix: tex_ix,
        tex_coords: [t_x, t_y]});

      // Other two vertices of the triangle
      data.push(Vertex {
        pos: [
          pos[0] + rad*(curr_angle.cos()), 
          pos[1] + rad*(curr_angle.sin())], 
        tex_type: TexType::Texture, tex_ix: tex_ix,
        col: col.clone(), tex_coords: [t_x, t_y]
      });
      data.push(Vertex {
        pos: [
          pos[0] + rad*((curr_angle+angle_increment).cos()), 
          pos[1] + rad*((curr_angle+angle_increment).sin())], 
        tex_type: TexType::Texture, tex_ix: tex_ix,
        col: col.clone(), tex_coords: [t_x, t_y]
      });

      // Increment the angle for the next loop
      curr_angle += angle_increment;
    }

    // Send the data
    self.sender.send(data).unwrap();
  }

  /// Render a texture.
  /// # Params 
  /// * `tex` - The handle of the texture to render.
  /// * `aabb` - The AABB bounding box of the final texture - X, Y, W, H.
  /// * `tint` - The colour to tint the texture.
  pub fn tex(&self, tex: TexHandle, aabb: &[f32; 4], 
             tint: &[f32; 4]) -> Result<(), RenderTextureError> {
    let (x, y, w, h) = (aabb[0], aabb[1], aabb[2], aabb[3]);
    let (tex_ix, rect) = try!(self.lookup_tex(tex).ok_or(RenderTextureError));

    let mut vertices = Vec::with_capacity(6);
    // Generate vertex data.
    vertices.push(Vertex {
      pos: [x, y],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[0], rect[3]],
    });
    vertices.push(Vertex {
      pos: [x + w, y],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[2], rect[3]],
    });
    vertices.push(Vertex {
      pos: [x + w, y + h],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[2], rect[1]],
    });
    vertices.push(Vertex {
      pos: [x, y],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[0], rect[3]],
    });
    vertices.push(Vertex {
      pos: [x, y + h],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[0], rect[1]],
    });
    vertices.push(Vertex {
      pos: [x + w, y + h],
      col: tint.clone(),
      tex_type: TexType::Texture, tex_ix: tex_ix,
      tex_coords: [rect[2], rect[1]],
    });

    self.sender.send(vertices).unwrap();
    return Ok(());
  }

  /// Render some text. 
  /// # Params
  /// * `text` - The text to render
  /// * `pos` - The position to render the text at - this is the bottom left of the first character.
  /// * `font_handle` - This is the font to render the text with.
  /// * `tint` - The tint to apply to the font.
  /// # Returns
  /// Error if not all the glyphs for this font were cached. To cache glyphs,
  /// use the cache_glyphs method on your QGFX instance.
  pub fn text(&self, text: &str, pos: &[f32; 2], 
              font_handle: FontHandle, tint: &[f32; 4]) -> Result<(), RenderTextError> {
    let font_cache = self.font_cache.lock().unwrap();
    let &(ref font, (scale, _)) = font_cache.get_font_ref(font_handle).unwrap();
    let mut vertices = Vec::with_capacity(text.len() * 6);
    let mut cursor = pos.clone();
    let mut last_glyph_id = None; // For kerning.
    for c in text.chars() {
      // Get the glyph metrics
      let glyph = try!(font_cache.get_glyph(font_handle, c).ok_or(RenderTextError));
      let h_metrics = glyph.unpositioned().h_metrics();
      let (x, y, w, h) = {
        let rect = glyph.pixel_bounding_box();
        if rect.is_some() {
          let rect = rect.unwrap();
          (rect.min.x as f32, rect.min.y as f32, 
           (rect.max.x - rect.min.x) as f32, (rect.max.y - rect.min.y) as f32)
        }
        else { (0.0, 0.0, 0.0, 0.0) }
      };

      let rect = try!(font_cache.rect_for(font_handle, c));
      // If none, just advance cursor and continue. Nothing to draw, but glyph
      // has dimensions
      if rect.is_none() { 
        cursor[0] += h_metrics.left_side_bearing;
        cursor[0] += h_metrics.advance_width;
        continue;
      }
      let rect = rect.unwrap();

      if last_glyph_id.is_some() {
        cursor[0] += font.pair_kerning(Scale::uniform(scale), last_glyph_id.unwrap(), glyph.id());
      }
      last_glyph_id = Some(glyph.id());

      cursor[0] += h_metrics.left_side_bearing;

      // Generate vertices
      vertices.push(Vertex {
        pos: [x + cursor[0], y + cursor[1]],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[0], rect[1]],
      });
      vertices.push(Vertex {
        pos: [x + cursor[0] + w, y + cursor[1]],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[2], rect[1]],
      });
      vertices.push(Vertex {
        pos: [x + cursor[0] + w, y + cursor[1] + h],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[2], rect[3]],
      });
      vertices.push(Vertex {
        pos: [x + cursor[0], y + cursor[1]],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[0], rect[1]],
      });
      vertices.push(Vertex {
        pos: [x + cursor[0], y + cursor[1] + h],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[0], rect[3]],
      });
      vertices.push(Vertex {
        pos: [x + cursor[0] + w, y + cursor[1] + h],
        col: tint.clone(),
        tex_type: TexType::Font, tex_ix: 0,
        tex_coords: [rect[2], rect[3]],
      });

      cursor[0] += h_metrics.advance_width;
    }

    self.sender.send(vertices).unwrap();
    return Ok(());
  }
}
