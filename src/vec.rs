//! A vector helper module, to avoid the nalgebra dep

use std::ops::Index;

#[derive(Clone, Copy, PartialEq)]
pub struct Vec2(pub [f32; 2]);

impl Vec2 {

  /// Return the length of this vector.
  pub fn len(&self) -> f32 { (self[0].powi(2) + self[1].powi(2)).sqrt() }

  /// Add another vector to this one and return the result.
  pub fn add(&self, other: Vec2) -> Vec2 { 
    Vec2([self[0] + other[0], self[1] + other[1]])
  }

  pub fn sub(&self, other: Vec2) -> Vec2 {
    self.add(other.mul(-1.0))
  }

  /// Multiply this vector by a value.
  pub fn mul(&self, factor: f32) -> Vec2 { 
    Vec2([self[0]*factor, self[1]*factor]) 
  }
  
  /// Divide this vector by a value. Like mul() but with 1/factor.
  pub fn div(&self, factor: f32) -> Vec2 { 
    self.mul(1.0/factor) 
  }

  /// Normalise this vector and return the result.
  pub fn nor(&self) -> Vec2 {
    self.div(self.len())
  }
}

impl Index<usize> for Vec2 {
  type Output = f32;
  fn index(&self, ix: usize) -> &Self::Output { &self.0[ix] }
}

