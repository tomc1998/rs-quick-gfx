use renderer::Vertex;
use std;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
use res::font::glium_cache::GliumFontCache;
use res::font::FontHandle;
use vec::Vec2;

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

/// This struct wraps a Sender<Vec<Vertex>>, and has convenience methods to
/// draw certain geometry.
#[derive(Clone, Debug)]
pub struct RendererController<'a> {
  font_cache: Arc<Mutex<GliumFontCache<'a>>>,
  sender: mpsc::Sender<Vec<Vertex>>,
}

impl<'a> RendererController<'a> {
  /// Creates a new renderer controller with a given mpsc sender. If you want
  /// to get a renderer controller, look at the
  /// renderer::Renderer::get_renderer_controller() function.
  pub fn new(sender: mpsc::Sender<Vec<Vertex>>, 
             font_cache: Arc<Mutex<GliumFontCache<'a>>>) -> Box<RendererController<'a>> {
    Box::new(RendererController { sender: sender, font_cache: font_cache })
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
    data.push(Vertex{ pos: [perp_l_1[0], perp_l_1[1]], col: col.clone(), tex_coords: [0.0, 0.0]});
    data.push(Vertex{ pos: [perp_r_1[0], perp_r_1[1]], col: col.clone(), tex_coords: [0.0, 0.0]});
    data.push(Vertex{ pos: [perp_l_2[0], perp_l_2[1]], col: col.clone(), tex_coords: [0.0, 0.0]});

    // tri 2
    data.push(Vertex{ pos: [perp_l_2[0], perp_l_2[1]], col: col.clone(), tex_coords: [0.0, 0.0]});
    data.push(Vertex{ pos: [perp_r_2[0], perp_r_2[1]], col: col.clone(), tex_coords: [0.0, 0.0]});
    data.push(Vertex{ pos: [perp_r_1[0], perp_r_1[1]], col: col.clone(), tex_coords: [0.0, 0.0]});

    // Send the vertex data through the sender
    self.sender.send(data).unwrap();
  }

  /// Draws a line given a start and an endpoint.
  /// #Params
  /// * `aabb` - The AABB box for the rectangle - X, Y, W, H
  /// * `col` - The colour of the rectangle
  pub fn rect(&self, aabb: &[f32; 4], col: &[f32; 4]) {
    let mut data = Vec::with_capacity(6);

    // Generate vertex data
    // Tri 1
    data.push( Vertex { pos: [aabb[0], aabb[1]], col: col.clone(), tex_coords: [0.0, 0.0] });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1]], col: col.clone(), tex_coords: [1.0, 0.0] });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], col: col.clone(), tex_coords: [1.0, 1.0] });

    // Tri 2
    data.push( Vertex { pos: [aabb[0], aabb[1]], col: col.clone(), tex_coords: [0.0, 0.0] });
    data.push( Vertex { pos: [aabb[0], aabb[1] + aabb[3]], col: col.clone(), tex_coords: [0.0, 1.0] });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], col: col.clone(), tex_coords: [1.0, 1.0] });

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
    let mut data = Vec::with_capacity(segments*3);
    let mut curr_angle = 0.0f32;
    let angle_increment = 2.0*(PI as f32)*(1.0 / segments as f32);
    for _ in 0..segments {
      // Vertex at the centre of the circle
      data.push(Vertex {pos: pos.clone(), col: col.clone(), tex_coords: [0.0, 0.0]});

      // Other two vertices of the triangle
      data.push(Vertex {
        pos: [
          pos[0] + rad*(curr_angle.cos()), 
          pos[1] + rad*(curr_angle.sin())], 
        col: col.clone(), tex_coords: [0.0, 0.0]
      });
      data.push(Vertex {
        pos: [
          pos[0] + rad*((curr_angle+angle_increment).cos()), 
          pos[1] + rad*((curr_angle+angle_increment).sin())], 
        col: col.clone(), tex_coords: [0.0, 0.0]
      });

      // Increment the angle for the next loop
      curr_angle += angle_increment;
    }

    // Send the data
    self.sender.send(data).unwrap();
  }

  /// Render some text. 
  /// # Params
  /// * `text` - The text to render
  /// * `pos` - The position to render the text at - this is the bottom left of the first character.
  /// * `font_handle` - This is the font to render the text with.
  /// # Returns
  /// Error if not all the glyphs for this font were cached. To cache glyphs,
  /// use the cache_glyphs method on your QGFX instance.
  pub fn text(&self, text: &str, pos: &[f32; 2], font_handle: FontHandle) -> Result<(), RenderTextError>{
    let font_cache = self.font_cache.lock().unwrap();
    for c in text.chars() {
      // Get the UV rect for this char
      let glyph = font_cache.get_glyph(font_handle, c);
      if glyph.is_none() { return Err(RenderTextError); }
      let glyph = glyph.unwrap();
      
    }
    return Ok(());
  }
}
