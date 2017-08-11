//! This is a module which contains a binary tree implementation for the bin
//! packing algorithm.

use std::fmt;
use std;
use res::tex::TexHandle;

#[derive(Clone, Copy, Debug)]
pub enum PackRectError {
  /// This variant is returned when the space in the node is too small for the
  /// given rect you're attempting to pack into it.
  SpaceTooSmall,
}
impl fmt::Display for PackRectError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use std::error::Error;
    write!(f, "{:?}", self.description())
  }
}
impl std::error::Error for PackRectError {
  fn description(&self) -> &'static str {
    match *self {
      PackRectError::SpaceTooSmall => 
      r#"This variant is returned when the space in the node is too small for
      the given rect you're attempting to pack into it."#,
    }
  }
}

/// A binary tree node used by the GliumTexCache.
pub struct BinaryTreeNode {
  l_child: Option<Box<BinaryTreeNode>>,
  r_child: Option<Box<BinaryTreeNode>>,

  /// The space contained in this node as a UV rect - XYWH
  space: [f32; 4],

  /// The texture handle associated with this space.
  tex_handle: Option<TexHandle>,
}
impl BinaryTreeNode {
  /// Create a new binary tree node with the given UV rect as space.
  pub fn new(space: [f32; 4]) -> BinaryTreeNode {
    BinaryTreeNode {
      l_child: None, r_child: None,
      space: space,
      tex_handle: None,
    }
  }

  /// Check if this node is a leaf. Useful to check if it stores an actual
  /// texture or not - if it's a leaf, it'll be empty.
  pub fn is_leaf(&self) -> bool {
    self.tex_handle.is_none()
  }

  /// Pack a rect into this space. Change this node into a branch, and add both
  /// children. l_child will be the rect below this newly packed rect, and
  /// r_child will be the remaining space to the right on the same row as the
  /// given rect.
  ///
  /// Free space on the right is a rectangle of the same height as the given
  /// rectangle to pack in this function. Space below has the same width,
  /// taking up the rest of the available height. This means the rectangle
  /// below this takes the 'diagonal' rectangle.
  /// # Params
  /// * `w` - The width of the rectangle in UV coordinates.
  /// * `h` - The height of the rectangle in UV coordinates.
  /// * `tex` - The texture handle of the texture we're packing.
  /// # Errors
  /// Returns an error if the given rect is too small for this space.
  /// # Notes
  /// If this node is not a leaf node, then this function will be recursively
  /// called on the child nodes of this node.
  fn pack_rect(&mut self, w: f32, h: f32, tex: TexHandle) -> Result<(), PackRectError> {
    if !self.is_leaf() {
      // Recurse.
      debug_assert!(self.l_child.is_some() && self.r_child.is_some(), 
                    r#"A node in the binary tree is a leaf, but for some reason
                    either l_child or r_child is not set."#);
      let res = self.r_child.as_mut().unwrap().pack_rect(w, h, tex);
      if res.is_err() {
        match res.err().unwrap() {
          PackRectError::SpaceTooSmall => return self.l_child.as_mut().unwrap().pack_rect(w, h, tex),
        }
      }
      else { return res; }
      unreachable!();
    }

    // Check the given w/h is small enough to fit
    if w > self.space[2] || h > self.space[3] {
      return Err(PackRectError::SpaceTooSmall);
    }

    // Calculate the space to the right and below once the rectangle has been
    // packed.
    let mut space_below = [0.0; 4];
    let mut space_right = [0.0; 4];
    space_below[0] = self.space[0];
    space_below[1] = self.space[1] + h;
    space_below[2] = self.space[2];
    space_below[3] = self.space[3] - h;
    space_right[0] = self.space[0] + w;
    space_right[1] = self.space[1];
    space_right[2] = self.space[2] - w;
    space_right[3] = self.space[3];

    // Create the child nodes
    self.l_child = Some(Box::new(BinaryTreeNode::new(space_below)));
    self.r_child = Some(Box::new(BinaryTreeNode::new(space_right)));

    // Set this node's space to the given rect, and the tex_handle
    self.space = [self.space[0], self.space[1], w, h];
    self.tex_handle = Some(tex);

    return Ok(());
  }
}
