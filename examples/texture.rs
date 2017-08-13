extern crate quick_gfx;

fn main() {
  // Create the renderer.
  let mut g = quick_gfx::QGFX::new();

  // Cache a texture
  let tex_handle = &g.cache_tex(&["rust-logo.png"])[0];
  if tex_handle.is_err() {
    println!("Error loading texture: {:?}", tex_handle.as_ref().err().unwrap());
  }
  let tex_handle = tex_handle.as_ref().unwrap().clone();

  // Get a controller, and draw the texture
  let controller = g.get_renderer_controller();

  // Draw a green rectangle at (0, 0) with dimensions (100, 100).
  controller.tex(tex_handle, &[0.0, 0.0, 512.0, 512.0], &[1.0, 1.0, 1.0, 1.0]).unwrap();

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

