use renderer::Vertex;
use std::sync::mpsc;
use std::ops::{Add, Sub, Mul};
use nalgebra::*;

/// This struct wraps a Sender<Vec<Vertex>>, and has convenience methods to
/// draw certain geometry.
#[derive(Clone, Debug)]
pub struct RendererController {
  sender: mpsc::Sender<Vec<Vertex>>,
}

impl RendererController {
  /// Creates a new renderer controller with a given mpsc sender. If you want
  /// to get a renderer controller, look at the
  /// renderer::Renderer::get_renderer_controller() function.
  pub fn new(sender: mpsc::Sender<Vec<Vertex>>) -> RendererController {
    RendererController { sender: sender, }
  }

  /// Draws a line given a start and an endpoint.
  /// #Params
  /// * `p1` - The starting point
  /// * `p2` - The ending point
  /// * `w` - The line width
  /// * `col` - The colour of the line
  pub fn line(&self, p1: Vector2<f32>, p2: Vector2<f32>, w: f32, col: [f32; 4]) {
    let mut data = Vec::with_capacity(6);
    let half_w = w/2.0;
    let p1p2 = p2.sub(p1);

    // Get the 4 corners of the 'rectangle' (the line is just a rectangle)
    let perp_l_1 = Vector2::new(-p1p2.y, p1p2.x).normalize().mul(half_w).add(p1);
    let perp_r_1 = Vector2::new(p1p2.y, -p1p2.x).normalize().mul(half_w).add(p1);
    let perp_l_2 = Vector2::new(-p1p2.y, p1p2.x).normalize().mul(half_w).add(p2);
    let perp_r_2 = Vector2::new(p1p2.y, -p1p2.x).normalize().mul(half_w).add(p2);

    // Generate the vertex data
    // tri 1
    data.push(Vertex{ pos: [perp_l_1.x, perp_l_1.y], col: col.clone()});
    data.push(Vertex{ pos: [perp_r_1.x, perp_r_1.y], col: col.clone()});
    data.push(Vertex{ pos: [perp_l_2.x, perp_l_2.y], col: col.clone()});

    // tri 2
    data.push(Vertex{ pos: [perp_l_2.x, perp_l_2.y], col: col.clone()});
    data.push(Vertex{ pos: [perp_r_2.x, perp_r_2.y], col: col.clone()});
    data.push(Vertex{ pos: [perp_r_1.x, perp_r_1.y], col: col.clone()});

    // Send the vertex data through the sender
    self.sender.send(data).unwrap();
  }

  /// Draws a line given a start and an endpoint.
  /// #Params
  /// * `aabb` - The AABB box for the rectangle - X, Y, W, H
  /// * `col` - The colour of the rectangle
  pub fn rect(&self, aabb: &[f32; 4], col: &[f32; 4]) {
    let mut data = Vec::with_capacity(6);

    // Generate vertex data
    // Tri 1
    data.push( Vertex { pos: [aabb[0], aabb[1]], col: col.clone() });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1]], col: col.clone() });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], col: col.clone() });

    // Tri 2
    data.push( Vertex { pos: [aabb[0], aabb[1]], col: col.clone() });
    data.push( Vertex { pos: [aabb[0], aabb[1] + aabb[3]], col: col.clone() });
    data.push( Vertex { pos: [aabb[0] + aabb[2], aabb[1] + aabb[3]], col: col.clone() });

    // Send the data
    self.sender.send(data).unwrap();
  }
}
