#[macro_use]
extern crate glium;
extern crate nalgebra;

mod renderer;

pub use renderer::RendererController;
pub use glium::backend::glutin_backend::PollEventsIter;
pub use glium::glutin::Event;

use glium::backend::glutin_backend::GlutinFacade;
use renderer::Renderer;


/// The API of the library.
pub struct QGFX {
  renderer: Box<Renderer>,
  display: GlutinFacade,
}

impl QGFX {
  /// Create a display with a renderer and return it. This function will open a window.
  pub fn new() -> QGFX {
    let display = init_display();
    let renderer = Renderer::new(&display);
    QGFX { 
      renderer: renderer,
      display: display 
    }
  }

  /// Get a renderer controller to send VBO data to this renderer. These can be
  /// cloned.
  pub fn get_renderer_controller(&self) -> RendererController {
    return self.renderer.get_renderer_controller();
  }

  /// Get the size of the display in pixels.
  /// # Returns
  /// An option containing the display size. Glium's methods can sometimes
  /// return none, but most likely the result will be Some and you needn't
  /// check for None before unwrapping.
  pub fn get_display_size(&self) -> Option<(i32, i32)> {
    let win = self.display.get_window();
    if win.is_none() { None }
    else { win.unwrap().get_position() }
  }

  /// Receive all the data sent by renderer controllers. This should be called
  /// before rendering to make sure the data is up to date.
  pub fn recv_data(&mut self) {
    self.renderer.recv_data();
  }

  pub fn render(&mut self) {
    use glium::Surface;
    let mut target = self.display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    self.renderer.render(&mut target);
    target.finish().unwrap();
  }

  pub fn poll_events(&self) -> PollEventsIter {
    self.display.poll_events()
  }
}

fn init_display() -> GlutinFacade {
    use glium::DisplayBuild;
    glium::glutin::WindowBuilder::new()
      .build_glium().unwrap()
}


