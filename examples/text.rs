extern crate quick_gfx;

use std::collections::HashSet;

fn main() {
  // Create the renderer, then cache some glyphs.
  let mut g = quick_gfx::QGFX::new();
  let mut charsets = HashSet::new();
  charsets.insert(quick_gfx::Charset::Lowercase);
  charsets.insert(quick_gfx::Charset::Uppercase);
  charsets.insert(quick_gfx::Charset::Numbers);
  charsets.insert(quick_gfx::Charset::Punctuation);
  let fh = g.cache_glyphs("Arial Unicode.ttf", 32.0, &quick_gfx::gen_charset(&charsets)[..]).unwrap();
  
  // Get a controller, and draw some text.
  let controller = g.get_renderer_controller();
  controller.text("The quick brown fox jumps over the lazy dog!",
                  &[100.0, 100.0], fh, &[1.0, 1.0, 1.0, 1.0]).unwrap();

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

