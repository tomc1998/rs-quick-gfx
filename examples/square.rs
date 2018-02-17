extern crate quick_gfx;

fn main() {
  // Create the renderer, and get a controller
  let mut g = quick_gfx::QGFX::new();


  {
    // The controller needs to be dropped before we do anything eles with the renderer, hence this
    // scoping.
    let mut controller = g.get_renderer_controller();

    // The controller is used to generate vertex buffer data and send it to the
    // renderer. To generate some data and send it to the renderer, call methods
    // on the controller:
    // Draw a green rectangle at (0, 0) with dimensions (256, 256).
    controller.rect(&[0.0, 0.0, 256.0, 256.0], &[0.0, 1.0, 0.0, 1.0]);
    controller.flush();
  }

  // Once we've send the data, we need to have the renderer receive it.
  g.recv_data();

  // Now that the renderer has the data, we can draw it.
  let mut closed = false;
  while !closed {
    // Poll events to check if window has been closed
    g.poll_events(|ev| {
      match ev {
        quick_gfx::Event::WindowEvent{event: ev, window_id: _} => {
          match ev {
            quick_gfx::WindowEvent::Closed => closed = true,
            _ => ()
          }
        },
        _ => ()
      }
    });

    // Render everything
    g.render();
  }
}
