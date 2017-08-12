mod shader;

/// A module containing the Controller class, an abstraction used to easily
/// send data to the renderer.
mod controller;

pub use self::controller::RendererController;

use std::path::Path;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
use glium::{self, VertexBuffer};
use res::font::glium_cache::GliumFontCache;
use res::font::{CacheGlyphError, FontHandle};
use res::tex::{CacheTexError, TexHandle};
use res::tex::glium_cache::GliumTexCache;

/// The constant size of the renderer's VBO in vertices (i.e. can contain 1024 vertices)
pub const VBO_SIZE : usize = 65563;

/// An enum for texture types. For example, when rendering a font, vertices
/// should be send with a 'Font' texture type, to indicate they will be drawn
/// with the font texture as the loaded uniform.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TexType {
  Texture, Font
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
  /// The position of the vertex. Sent to the shader.
  pub pos: [f32; 2],
  /// The UV coordinates of the vertex. Sent to the shader.
  pub tex_coords: [f32; 2],
  /// The colour of this vertex. Sent to the shader.
  pub col: [f32; 4],
  /// The type of texture this vertex will use. See enum TexType. NOT sent to the shader.
  pub tex_type: TexType,
  /// The index of the texture in the cache to use. Texture caches can have
  /// multiple textures stored in video ram, this number indicates which to
  /// use. NOT sent to the shader.
  pub tex_ix: usize,
}
implement_vertex!(Vertex, pos, tex_coords, col);

pub struct Renderer<'a> {
  /// The VBO to use. This will have data buffered to it when render() is called.
  vbo: VertexBuffer<Vertex>,

  /// The program to use for rendering
  program: glium::Program,

  /// The vertex data to be draw when render() is called. Data is moved into
  /// this buffer when `recv_data()` is called, then moved to the VBO for
  /// rendering in `render()`.
  v_data: Vec<Vertex>,

  /// A tuple containing a sender and receiver - used for sending data to
  /// the renderer from different threads to be stored in v_data for the
  /// render() function.
  v_channel_pair: (mpsc::Sender<Vec<Vertex>>, mpsc::Receiver<Vec<Vertex>>),

  /// The projection matrix used to render the game. 
  proj_mat: [[f32; 4]; 4],

  font_cache: Arc<Mutex<GliumFontCache<'a>>>,
  tex_cache: Arc<Mutex<GliumTexCache>>,
}

impl<'a> Renderer<'a>{
  /// Create a new renderer.
  /// # Params
  /// * `display` - The glutin display (OpenGL Context)
  /// * `system` - The SysRenderer being used by the ECS. When rendering,
  ///              vertex data will be buffered from here.
  pub fn new(display: &glium::Display) -> Box<Renderer<'a>> {
    let (w, h) = display.get_framebuffer_dimensions();
    let font_cache = GliumFontCache::new(display);
    Box::new(Renderer {
      vbo: VertexBuffer::empty_dynamic(display, VBO_SIZE).unwrap(),
      program: shader::get_program(display),
      v_data: Vec::new(),
      v_channel_pair: mpsc::channel(),
      font_cache: Arc::new(Mutex::new(font_cache)),
      tex_cache: Arc::new(Mutex::new(GliumTexCache::new())),
      proj_mat: [[2.0/w as f32, 0.0,           0.0, -0.0],
                 [0.0,         -2.0/h as f32,  0.0,  0.0],
                 [0.0,          0.0,          -1.0,  0.0],
                 [-1.0,         1.0,           0.0,  1.0]],
    })
  }

  /// Buffer the vertex data received from the ECS render system
  /// (`SysRenderer`) to the VBO to be rendered. This should be called before
  /// `render()`.
  pub fn recv_data(&mut self) {
    self.v_data.clear();
    // VBO_SIZE, no more data must be buffered.
    loop {
      let res = self.v_channel_pair.1.try_recv();
      if res.is_err() {
        // If the result of try_recv is an error, either all the sender's are
        // disconnected (not expected, as we own a sender) OR the channel is
        // empty, which means we've buffered all the data we can.
        match res.err().unwrap() {
          mpsc::TryRecvError::Empty => break,
          mpsc::TryRecvError::Disconnected => panic!("Vertex data senders disconnected!")
        }
      }
      // Copy data from the packet into v_data
      let data_packet = res.unwrap();

      for v in data_packet {
        self.v_data.push(v);

        // Check data packet won't be too long
        #[cfg(feature = "vbo_overflow_panic")]
        { if self.v_data.len() >= VBO_SIZE { panic!("VBO Overflow"); } }
      }
    }

    while self.v_data.len() < VBO_SIZE {
      self.v_data.push(Vertex { 
        pos: [0.0; 2], col: [0.0; 4], 
        tex_coords: [0.0, 0.0], 
        tex_ix: 0, tex_type: TexType::Texture} );
    }
  }

  pub fn render<T : glium::Surface>(&mut self, target: &mut T) {

    // Empty indices - basically only rendering sprites, so no need to have it indexed.
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // Write the vertex data to the VBO
    self.vbo.write(&self.v_data);

    let font_cache = self.font_cache.lock().unwrap();
    let tex = font_cache.get_tex();

    // Load the uniforms
    let uniforms = uniform! {
      proj_mat: self.proj_mat,
      tex: tex,
    };

    // Draw everything!
    target.draw(&self.vbo, 
                &indices, 
                &self.program, 
                &uniforms, 
                &glium::DrawParameters {
                  blend: glium::Blend::alpha_blending(),
                  .. Default::default()
                }).unwrap();
  }

  /// # Returns
  /// A Sender<Vertex> for sending vertex data to the renderer. When
  /// render() is called, this data will be rendered then cleared.
  pub fn get_renderer_controller(&self) -> Box<RendererController<'a>> {
    RendererController::new(self.v_channel_pair.0.clone(), 
                            self.font_cache.clone(), 
                            self.tex_cache.clone())
  }

  /// A function to add the given chars to the cache. See res::font::FontCache
  /// for more details. This wraps the font_cache stored inside the renderer.
  /// This locks the mutex on the font cache, so any font rendering or caching
  /// on other threads will also be blocked for the duration.
  pub fn cache_glyphs<F: AsRef<Path>>(&self, file: F, scale: f32, 
                                      charset: &[char]) -> Result<FontHandle, CacheGlyphError> {
    use res::font::FontCache;
    self.font_cache.lock().unwrap().cache_glyphs(file, scale, charset)
  }

  /// Cache textures from filepaths, returning a list of texture handles.
  pub fn cache_tex<F: AsRef<Path>>(
    &self, display: &glium::Display, 
    filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>> {
    use res::tex::TexCache;
    self.tex_cache.lock().unwrap().cache_tex(display, filepaths)
  }
}

