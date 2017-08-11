//! A module containing a glium implementation of a tex cache.

use glium::texture::Texture2d;
use res::tex::*;
use std::collections::BTreeMap;

/// Texture cache which uses glium as the GPU storage medium.
pub struct GliumTexCache {
  /// The maximum amount of cache textures to be created.
  max_cache_textures: usize,

  /// The size of the GPU cache textures.
  cache_texture_size: (u32, u32),

  /// The list of cache textures.
  cache_textures: Vec<Texture2d>,

  /// A map of texture handles to UV coordinates and a cache texture ID.
  tex_map: BTreeMap<TexHandle, (usize, [f32; 4])>
}

impl TexCache for GliumTexCache {
#[allow(unused_variables)]
  fn cache_tex<F: AsRef<Path>>(&mut self, filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>> {
    unimplemented!();
  }

#[allow(unused_variables)]
  fn free_tex(&mut self, tex: &[TexHandle]) {
    unimplemented!();
  }

  fn is_tex_cached(&self, tex: TexHandle) -> bool {
    self.tex_map.contains_key(&tex)
  }

  fn rect_for(&self, tex: &[TexHandle]) -> Vec<Option<(usize, [f32; 4])>> {
    let mut result = Vec::with_capacity(tex.len());
    for t in tex {
      result.push(self.tex_map.get(&t).cloned());
    }
    return result;
  }

  fn get_tex_with_ix(&self, ix: usize) -> Option<&Texture2d> {
    if self.cache_textures.len() <= ix { None }
    else { Some(&self.cache_textures[ix]) }
  }

  fn set_max_cache_textures(&mut self, max_cache_textures: usize) {
    self.max_cache_textures = max_cache_textures;
  }

  fn set_cache_texture_size(&mut self, w: u32, h: u32) {
    self.cache_texture_size = (w, h);
  }
}
