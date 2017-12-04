pub mod glium_cache;

use glium;
use std;
use image;
use std::path::Path;
use glium::texture::srgb_texture2d::SrgbTexture2d;

/// A texture handle. This references a texture loaded into the cache.
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
pub struct TexHandle(usize);

/// An error returned when caching a texture.
#[derive(Debug)]
pub enum CacheTexError {
  /// Returned when the texture itself is too big to fit in the cache. This
  /// means that regardless of even if the cache was empty, the texture is too
  /// big.
  CacheTooSmall,

  /// Returned when there's not enough space for this texture in the cache
  /// without removing others, or adding another cache texture. Use
  /// set_max_cache_textures to increase the number of cache textures to
  /// accommodate for this texture.
  NoSpace,

  /// An IO error occurred when reading the texture file.
  IoError(std::io::Error),

  /// An error occurred creating an image from the bytes read.
  ImageError(image::ImageError),

  /// The cache tried to create a texture which was too large to be supported.
  DimensionsNotSupported,
}

/// A trait for a GPU texture cache.
pub trait TexCache {
  /// A function to cache some textures and return texture handles.
  /// 
  /// Texture handles are returned in a slice with the indexes corresponding to
  /// the indexes in the slice of texture files given.
  fn cache_tex<F: AsRef<Path>>(
    &mut self, display: &glium::Display, 
    filepaths: &[F]) -> Vec<Result<TexHandle, CacheTexError>>;

  /// A function to cache some textures and return texture handles.
  /// 
  /// Texture handles are returned in a slice with the indexes corresponding to
  /// the indexes in the slice of texture files given.
  fn cache_tex_from_bytes(
    &mut self, display: &glium::Display, 
    bytes: &[&[u8]]) -> Vec<Result<TexHandle, CacheTexError>>;

  /// A function to free a given list of texture from the cache. If a
  /// texture is not cached, it is ignored.
  fn free_tex(&mut self, tex: &[TexHandle]);

  /// Returns true if the given texture handle is cached.
  fn is_tex_cached(&self, tex: TexHandle) -> bool;

  /// Returns texture coordinate rectangles and texture indexes for the
  /// location of the given textures in the cache. Similar to the cache_tex
  /// function, results in the returned vector match the indexes given. 
  /// 
  /// To find the texture given by the texture index, use get_tex_with_ix(). 
  ///
  /// If the texture is not cached, this function returns None in its place in
  /// the returned array.
  fn rect_for(&self, tex: TexHandle) -> Option<(usize, [f32; 4])>;

  /// Gets a reference to the cache texture with the given index. If the
  /// texture is not found, returns None.
  fn get_tex_with_ix(&self, ix: usize) -> Option<&SrgbTexture2d>;

  /// Sets the maximum amount of cache textures to create. 0 means limitless.
  /// If you put a cap on the amount of textures that can be used to cache on
  /// the GPU, you may get CacheTexError::NoSpace returned when you call
  /// cache_tex.
  fn set_max_cache_textures(&mut self, max_cache_textures: usize);

  /// Sets the size of cache textures. Bigger sizes may not be supported on
  /// some GPUs, but smaller sizes will result in more draw calls for
  /// applications with lots of textures.
  fn set_cache_texture_size(&mut self, w: u32, h: u32);
}
