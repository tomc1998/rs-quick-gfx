extern crate quick_gfx;

use std::cmp::min;

fn main() {
  // Create the renderer and get a controller
  let mut qgfx = quick_gfx::QGFX::new();
  let controller = qgfx.get_renderer_controller();

  // Get display size
  let (mut win_w, mut win_h) = qgfx.get_display_size().unwrap();

  // Keep track of the ball's position, vel, and ball_radius (radius changes when screen size changes)
  let mut ball_pos = [win_w as f32 / 2.0, win_h as f32 / 2.0];
  let mut ball_vel = [10.0, 0.0];
  let mut ball_rad = min(win_w / 10, win_h / 10) as f32;

  // How much gravity there is (number which is applied to velocity every loop
  const GRAVITY : f32 = 1.0;

  loop {
    // Check whether the user requested a close or if the display size has
    // changed
    for ev in qgfx.poll_events() {
      match ev {
        quick_gfx::Event::Closed => return,
        quick_gfx::Event::Resized(new_w, new_h) => {
          // Window size has changed, reset the ball_radius and position of the ball
          ball_pos = [new_w as f32 / 2.0, new_h as f32 / 2.0];
          ball_rad = min(new_w / 10, new_h / 10) as f32;
          // Update win size
          win_w = new_w as i32;
          win_h = new_h as i32;
        }
        _ => ()
      }
    }

    // Apply ball pos / vel
    ball_vel[1] += GRAVITY;
    ball_pos[0] += ball_vel[0];
    ball_pos[1] += ball_vel[1] + 0.5*GRAVITY;

    // Check for collisions
    if ball_pos[0] - ball_rad < 0.0 {
      ball_vel[0] = -ball_vel[0];
      ball_pos[0] = ball_rad;
    }
    if ball_pos[0] + ball_rad > win_w as f32 {
      ball_vel[0] = -ball_vel[0];
      ball_pos[0] = win_w as f32 - ball_rad;
    }
    if ball_pos[1] - ball_rad < 0.0 {
      ball_vel[1] = -ball_vel[1];
      ball_pos[1] = ball_rad;
    }
    if ball_pos[1] + ball_rad > win_h as f32 {
      ball_vel[1] = -ball_vel[1];
      ball_pos[1] = win_h as f32 - ball_rad;
    }

    // Render the circle
    controller.circle(&ball_pos, ball_rad, 32, &[1.0, 0.0, 1.0, 0.0]);
    qgfx.recv_data();
    qgfx.render();
  }
}
