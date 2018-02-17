//! A module containing a glium implementation of a tex cache.

use glium;
use glium::texture::{RawImage2d};
use glium::texture::srgb_texture2d::SrgbTexture2d;
use res::tex::*;
use image;
use std::sync::Arc;

mod binary_tree;

use self::binary_tree::{BinaryTreeNode, BinaryTree};

pub type GliumTexHandleLookup = Arc<BinaryTree>;

/// Texture cache which uses glium as the GPU storage medium.
pub struct GliumTexCache {
  /// The maximum amount of cache textures to be created.
  max_cache_textures: usize,

  /// The size of the GPU cache textures.
  cache_texture_size: (u32, u32),

  /// The list of cache textures.
  cache_textures: Vec<SrgbTexture2d>,

  /// This is a list of root nodes for binary trees. They're used to pack
  /// textures into the cache. Each index in this vector matches a cache
  /// texture of the same index.
  bin_pack_trees: Arc<BinaryTree>,

  /// This field holds the value of the next valid TexHandle to hand out.
  next_tex_handle: TexHandle,
}

impl GliumTexCache {
  pub fn new() -> GliumTexCache {
    GliumTexCache {
      max_cache_textures: 0,
      cache_texture_size: (2048, 2048),
      cache_textures: Vec::new(),
      bin_pack_trees: Arc::new(Vec::new()),
      next_tex_handle: TexHandle(0),
    }
  }

  /// Gets a reference to the internal binary tree for bin packing, which supports texture UV
  /// lookup whilst also being send and sync.
  pub fn get_tex_lookup(&self) -> GliumTexHandleLookup {
      self.bin_pack_trees.clone()
  }

  fn get_next_tex_handle(&mut self) -> TexHandle {
    let th = self.next_tex_handle;
    self.next_tex_handle.0 += 1;
    return th;
  }

  /// The method to actually internally cache textures. Called by both of the
  /// caching methods implemented when implementing the TexCache trait.
  fn cache_tex_internal<F: glium::backend::Facade>(
    &mut self, display: &F, 
    bytes: Vec<Result<&[u8], CacheTexError>>) -> Vec<Result<TexHandle, CacheTexError>> {
    let mut result = Vec::with_capacity(bytes.len());
    for buf in bytes {
      if buf.is_err() { 
        result.push(Err(buf.err().unwrap()));
        continue;
      }
      let buf = buf.unwrap();
      // Load into an actual 'image',       
      let img = image::load_from_memory(buf);
      if img.is_err() {
        result.push(Err(CacheTexError::ImageError(img.err().unwrap())));
        continue;
      }
      let img = img.unwrap().to_rgba();

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
      let mut rect = None;
      let bin_pack_trees = Arc::get_mut(&mut self.bin_pack_trees)
        .expect("Failed to acquire mutable reference when caching texture. Is the texture cache in
                use?");
      for (ii, t) in bin_pack_trees.iter_mut().enumerate() {
        let res = t.pack_rect(w as f32 / self.cache_texture_size.0 as f32, 
                              h as f32 / self.cache_texture_size.1 as f32, 
                              tex_handle);
        if res.is_ok() { tex_ix = Some(ii); rect = Some(res.unwrap()); break; }
      }

      // If we haven't managed to pack the texture into existing cache
      // textures, then we need to create a new texture2d.
      if tex_ix.is_none() {
        if self.max_cache_textures > 0 && 
          self.cache_textures.len() >= self.max_cache_textures {
            result.push(Err(CacheTexError::NoSpace));
            continue;
          }

        use std::borrow::Cow;
        let data_len = self.cache_texture_size.0 as usize 
          * self.cache_texture_size.1 as usize;
        let mut data = Vec::with_capacity(data_len*4);
        data.resize(data_len*4, 0.0);
        let tex = SrgbTexture2d::new(display, RawImage2d {
          data: Cow::Owned(data),
          width: self.cache_texture_size.0,
          height: self.cache_texture_size.1,
          format: glium::texture::ClientFormat::F32F32F32F32,
        });
        if tex.is_err() {
          match tex.err().unwrap() {
            glium::texture::TextureCreationError::DimensionsNotSupported => {
              result.push(Err(CacheTexError::DimensionsNotSupported));
              continue;
            }
            e => panic!("Unexpected error when creating cache texture: {}", e),
          }
        }
        self.cache_textures.push(tex.unwrap());
        bin_pack_trees.push(BinaryTreeNode::new([0.0, 0.0, 1.0, 1.0]));

        // Pack the rect into this new texture.  No need to error handle this
        // one, too small error handled earlier in this function
        rect = Some(bin_pack_trees.last_mut().unwrap().pack_rect( 
            w as f32 / self.cache_texture_size.0 as f32, 
            h as f32 / self.cache_texture_size.1 as f32, 
            tex_handle).unwrap());
        tex_ix = Some(self.cache_textures.len() - 1);
      }

      // Actually buffer to the GPU.
      let tex_ix = tex_ix.unwrap();
      let rect = rect.unwrap();
      self.cache_textures[tex_ix].main_level().write(glium::Rect {
        left: (self.cache_texture_size.0 as f32 * rect[0]) as u32,
        bottom: (self.cache_texture_size.1 as f32 * rect[1]) as u32,        
        width: (self.cache_texture_size.0 as f32 * rect[2]) as u32,        
        height: (self.cache_texture_size.1 as f32 * rect[3]) as u32,      
      }, glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), (w, h)));

      result.push(Ok(tex_handle));
    }

    return result;
  }
}

impl TexCache for GliumTexCache {
  fn cache_tex<F: AsRef<Path>, Facade: glium::backend::Facade>(
    &mut self, display: &Facade, 
    filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>> {
    use std::fs::File;
    use std::io::Read;
    let mut result = Vec::with_capacity(filepaths.len());
    let mut bufs = Vec::with_capacity(filepaths.len());
    bufs.resize(filepaths.len(), Vec::new());

    // Load all the textures given.
    for (ii, f) in filepaths.iter().enumerate() {
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
      bufs.insert(ii, buf);
      result.push(Ok(())); 
    }

    let mut result_slices = Vec::with_capacity(filepaths.len());
    for (ii, r) in result.into_iter().enumerate() {
      if r.is_ok() { result_slices.push(Ok(bufs[ii].as_slice())); }
      else { result_slices.push(Err(r.unwrap_err())); }
    }

    // Need to map the owned data into slices now.
    self.cache_tex_internal(display, result_slices)
  }

  /// This must be called on the main thread, with the GL context as it may
  /// create textures (this is enforced by the need to pass in the
  /// glium::Display).
  fn cache_tex_from_bytes<F: glium::backend::Facade>(
    &mut self, display: &F, 
    bytes: &[&[u8]]) -> Vec<Result<TexHandle, CacheTexError>> {
    let vec : Vec<Result<&[u8], CacheTexError>> 
      = bytes.iter().map(|buf| Ok(*buf)).collect();
    self.cache_tex_internal(display, vec)
  }

#[allow(unused_variables)]
  fn free_tex(&mut self, tex: &[TexHandle]) {
    unimplemented!();
  }

  fn get_tex_with_ix(&self, ix: usize) -> Option<&SrgbTexture2d> {
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

impl TexHandleLookup for GliumTexCache {
  fn is_tex_cached(&self, tex: TexHandle) -> bool {
    self.rect_for(tex).is_some()
  }

  fn rect_for(&self, tex: TexHandle) -> Option<(usize, [f32; 4])> {
    for (ii, t) in self.bin_pack_trees.iter().enumerate() {
      let res = t.rect_for(tex);
      if res.is_some() { return Some((ii, res.unwrap())); };
    }
    return None;
  }
}

