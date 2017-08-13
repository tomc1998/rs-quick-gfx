#[macro_use]
extern crate glium;
extern crate winit;
extern crate rusttype;
extern crate image;

mod renderer;
mod vec;
mod res;

pub use renderer::RendererController;
pub use glium::glutin::Event;
pub use glium::glutin::WindowEvent;
pub use glium::glutin::DeviceEvent;
pub use winit::{VirtualKeyCode, ElementState};
pub use res::font::{gen_charset, Charset};

use glium::Display;
use glium::glutin::EventsLoop;
use renderer::Renderer;
use std::sync::Mutex;
use std::path::Path;
pub use res::font::{FontHandle, CacheGlyphError};
pub use res::tex::{TexHandle, CacheTexError};


/// The API of the library.
pub struct QGFX<'a> {
  renderer: Box<Renderer<'a>>,
  display: Display,
  events_loop: Mutex<EventsLoop>,
  /// A tex handle for a 1x1 white texture. Used when rendering colours.
  white_tex_handle: TexHandle,
}

impl<'a> QGFX<'a> {
  /// Create a display with a renderer and return it. This function will open a window.
  pub fn new() -> QGFX<'a> {
    let (display, events_loop) = init_display();
    let renderer = Renderer::new(&display);

    // We need to buffer a small white rectangle, for when drawing coloured
    // shapes. The following is an array for a bitmap with a 1x1 white pixel.
    let bytes = [0x42, 0x4d, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x3e, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x01, 0x00,
                 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
                 0xff, 0x00, 0x80, 0x00, 0x00, 0x00];
    let t_vec_ref = &renderer.cache_tex_from_bytes(&display, &[&bytes[..]])[0];
    if t_vec_ref.is_err() {
      println!("{:?}", t_vec_ref.as_ref().err().unwrap());
    }
    let white_tex_handle = t_vec_ref.as_ref().unwrap();

    QGFX { 
      renderer: renderer,
      display: display,
      events_loop: Mutex::new(events_loop),
      white_tex_handle: white_tex_handle.clone(),
    }
  }

  /// Get a renderer controller to send VBO data to this renderer. These can be
  /// cloned.
  pub fn get_renderer_controller(&self) -> Box<RendererController<'a>> {
    return self.renderer.get_renderer_controller(self.white_tex_handle);
  }

  /// Cache some glyphs from a font.
  pub fn cache_glyphs<F: AsRef<Path>> (
    &self, file: F, scale: f32, 
    charset: &[char]) -> Result<FontHandle, CacheGlyphError> {
    self.renderer.cache_glyphs(file, scale, charset)
  }

  /// A function to cache some textures and return texture handles.
  /// 
  ///
  /// # Params
  /// * `filepaths` - The list of textures as filepaths.
  ///
  /// # Returns
  /// A list of texture handles, corresponding to textures on the GPU. Texture
  /// handles are returned in a slice with the indices corresponding to the
  /// indices in the slice of texture files given.
  ///
  /// # Errors
  /// Each texture may cause an error separately. Errors may occur if a texture
  /// is too big for the texture cache, or if there was an error loading the
  /// image etc.
  pub fn cache_tex<F: AsRef<Path>>(&self, filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>> {
    self.renderer.cache_tex(&self.display, filepaths)
  }

  pub fn cache_tex_from_bytes(&self, bytes: &[&[u8]]) -> Vec<Result<TexHandle, CacheTexError>> {
    self.renderer.cache_tex_from_bytes(&self.display, bytes)
  }

  /// Get the size of the display in pixels.
  pub fn get_display_size(&self) -> (u32, u32) {
    self.display.get_framebuffer_dimensions()
  }

  /// Receive all the data sent by renderer controllers. This should be called
  /// before rendering to make sure the data is up to date.
  pub fn recv_data(&mut self) {
    self.renderer.recv_data();
  }

  pub fn render(&mut self) {
    use glium::Surface;
    let mut target = self.display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    self.renderer.render(&mut target);
    target.finish().unwrap();
  }

  /// Poll events on this window. If there are any events available, call the
  /// provided callback F with the given event as an argument.
  /// This will lock the events loop inside this structure. It will panic if
  /// the mutex lock is poisoned. This is intentional (Rather a panic than
  /// something as crucial as an event loop erroring silently).
  pub fn poll_events<F: FnMut(Event) -> ()>(&self, callback: F) {
    self.events_loop.lock().unwrap().poll_events(callback)
  }
}

fn init_display() -> (Display, EventsLoop) {
  // 1. The **winit::EventsLoop** for handling events.
  let events_loop = glium::glutin::EventsLoop::new();

  // 2. Parameters for building the Window.
  let window = glium::glutin::WindowBuilder::new()
    .with_dimensions(1024, 768)
    .with_title("Hello world");

  // 3. Parameters for building the OpenGL context.
  let context = glium::glutin::ContextBuilder::new();

  // 4. Build the Display with the given window and OpenGL context parameters and register the
  //    window with the events_loop.
  (glium::Display::new(window, context, &events_loop).unwrap(), events_loop)
}


