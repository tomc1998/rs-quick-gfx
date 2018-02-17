#![cfg(test)]

use glium;
use glium::glutin::*;

/// Create a new headless 800 x 600 display
pub fn create_headless_display() -> glium::backend::glutin::headless::Headless {
    glium::backend::glutin::headless::Headless::new(
        HeadlessRendererBuilder::new(800, 600)
            .build()
            .unwrap(),
    ).unwrap()
}
