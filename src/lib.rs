#[macro_use]
extern crate glium;
extern crate winit;
extern crate rusttype;

mod renderer;
mod vec;
mod res;

pub use renderer::RendererController;
pub use glium::glutin::Event;
pub use glium::glutin::WindowEvent;
pub use glium::glutin::DeviceEvent;
pub use winit::{VirtualKeyCode, ElementState};

use glium::Display;
use glium::glutin::EventsLoop;
use renderer::Renderer;
use std::sync::Mutex;


/// The API of the library.
pub struct QGFX<'a> {
  renderer: Box<Renderer<'a>>,
  display: Display,
  events_loop: Mutex<EventsLoop>,
}

impl<'a> QGFX<'a> {
  /// Create a display with a renderer and return it. This function will open a window.
  pub fn new() -> QGFX<'a> {
    let (display, events_loop) = init_display();
    let renderer = Renderer::new(&display);
    renderer.cache_glyphs("Arial Unicode.ttf", 24.0, &['a', 'b', 'c'][..]).unwrap();
    QGFX { 
      renderer: renderer,
      display: display,
      events_loop: Mutex::new(events_loop),
    }
  }

  /// Get a renderer controller to send VBO data to this renderer. These can be
  /// cloned.
  pub fn get_renderer_controller(&self) -> Box<RendererController<'a>> {
    return self.renderer.get_renderer_controller();
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
    target.clear_color(0.0, 0.0, 0.0, 1.0);
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


