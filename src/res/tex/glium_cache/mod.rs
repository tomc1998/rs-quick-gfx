//! A module containing a glium implementation of a tex cache.

use glium;
use glium::texture::Texture2d;
use res::tex::*;
use image;

mod binary_tree;

use self::binary_tree::BinaryTreeNode;

/// Texture cache which uses glium as the GPU storage medium.
pub struct GliumTexCache {
  /// The maximum amount of cache textures to be created.
  max_cache_textures: usize,

  /// The size of the GPU cache textures.
  cache_texture_size: (u32, u32),

  /// The list of cache textures.
  cache_textures: Vec<Texture2d>,

  /// This is a list of root nodes for binary trees. They're used to pack
  /// textures into the cache. Each index in this vector matches a cache
  /// texture of the same index.
  bin_pack_trees: Vec<BinaryTreeNode>,

  /// This field holds the value of the next valid TexHandle to hand out.
  next_tex_handle: TexHandle,
}

impl GliumTexCache {
  pub fn new() -> GliumTexCache {
    GliumTexCache {
      max_cache_textures: 0,
      cache_texture_size: (2048, 2048),
      cache_textures: Vec::new(),
      bin_pack_trees: Vec::new(),
      next_tex_handle: TexHandle(0),
    }
  }

  fn get_next_tex_handle(&mut self) -> TexHandle {
    let th = self.next_tex_handle;
    self.next_tex_handle.0 += 1;
    return th;
  }
}

impl TexCache for GliumTexCache {
  /// This must be called on the main thread, with the GL context as it may create textures.
#[allow(unused_variables)]
  fn cache_tex<F: AsRef<Path>>(&mut self, display: &glium::Display, filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>> {
    use std::fs::File;
    use std::io::Read;
    let mut result = Vec::with_capacity(filepaths.len());
    // Load all the textures given.
    for f in filepaths {
      // Try open the file
      let file = File::open(f);
      if file.is_err() {
        result.push(Err(CacheTexError::IoError(file.err().unwrap())));
        continue;
      }
      let mut file = file.unwrap();

      // Read all the file data
      let mut buf = Vec::new();
      let read_res = file.read_to_end(&mut buf);
      if read_res.is_err() {
        result.push(Err(CacheTexError::IoError(read_res.err().unwrap())));
        continue;
      }

      // Load into an actual 'image',       
      use image::GenericImage;
      let img = image::load_from_memory(buf.as_slice());
      if img.is_err() {
        result.push(Err(CacheTexError::ImageError(img.err().unwrap())));
        continue;
      }
      let img = img.unwrap();

      // Check if the cache tex size is big enough to contain this texture.
      let (w, h) = img.dimensions();
      if w > self.cache_texture_size.0 || h > self.cache_texture_size.1 {
        result.push(Err(CacheTexError::CacheTooSmall));
        continue;
      }

      let tex_handle = self.get_next_tex_handle();
      // Now try and fit it into the cache using the bin packing algorithm.
      // Loop over all the current textures and try to pack_rect.
      let mut tex_ix = None;
      for (ii, t) in self.bin_pack_trees.iter_mut().enumerate() {
        let res = t.pack_rect(w as f32 / self.cache_texture_size.0 as f32, 
                              h as f32 / self.cache_texture_size.1 as f32, 
                              tex_handle);
        if res.is_ok() { tex_ix = Some(ii); break; }
      }
      // If we haven't managed to pack the texture into existing cache
      // textures, then we need to create a new texture2d.
      if tex_ix.is_none() {
      }
    }

    return result;
  }

#[allow(unused_variables)]
  fn free_tex(&mut self, tex: &[TexHandle]) {
    unimplemented!();
  }

#[allow(unused_variables)]
  fn is_tex_cached(&self, tex: TexHandle) -> bool {
    unimplemented!();
  }

#[allow(unused_variables)]
  fn rect_for(&self, tex: &[TexHandle]) -> Vec<Option<(usize, [f32; 4])>> {
    unimplemented!();
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

