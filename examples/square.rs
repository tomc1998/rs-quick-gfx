extern crate quick_gfx;

fn main() {
  // Create the renderer, and get a controller
  let mut g = quick_gfx::QGFX::new();
  let controller = g.get_renderer_controller();

  // The controller is used to generate vertex buffer data and send it to the
  // renderer. To generate some data and send it to the renderer, call methods
  // on the controller:

  // Draw a green rectangle at (0, 0) with dimensions (100, 100).
  controller.rect(&[0.0, 0.0, 100.0, 100.0], &[0.0, 1.0, 0.0, 1.0]);

  // Once we've send the data, we need to have the renderer receive it.
  g.recv_data();

  // Now that the renderer has the data, we can draw it.
  loop {
    // Poll events to check if window has been closed
    for ev in g.poll_events() {
      match ev {
        quick_gfx::Event::Closed => return,
        _ => ()
      }
    }

    // Render everything
    g.render();
  }
}
