use std;
use rusttype::gpu_cache::CacheReadErr;
use std::ops::Deref;
use std::path::{PathBuf, Path};
use std::collections::HashSet;
use std::fmt::{Display, Formatter, self};
use rusttype::{PositionedGlyph, Font};

pub mod glium_cache;

/// An error enum returned by the cache_glyphs() function in the FontCache
/// trait.
#[derive(Debug)]
pub enum CacheGlyphError {
  /// Error occurs if one or more chars in the given charset is not supported
  /// by the font. Contains the list of chars which were not supported by the
  /// font.
  GlyphNotSupported(Vec<char>),

  /// Error returned when the cache is too small to accommodate all the
  /// characters listed in the charset.
  CacheTooSmall,

  /// An IO error occurred when reading the font file.
  IoError(std::io::Error),
}

impl Display for CacheGlyphError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match *self {
      CacheGlyphError::GlyphNotSupported(ref chars) => 
        write!(f, r#"The following chars are not supported by the given font:
               {:?}"#, chars),
      CacheGlyphError::CacheTooSmall => 
        write!(f, r#"The cache is to small to contain all the characters
             given."#),
      CacheGlyphError::IoError(ref e) => write!(f, "{}", e),
    }
  }
}

impl std::error::Error for CacheGlyphError {
  fn description(&self) -> &str { 
    match *self {
      CacheGlyphError::GlyphNotSupported(_) => "A glyph is not supported.",
      CacheGlyphError::CacheTooSmall => "The cache is too small for these characters with this font.",
      CacheGlyphError::IoError(ref e) => e.description(),
    }
  }
}

impl std::convert::From<std::io::Error> for CacheGlyphError {
  fn from(e: std::io::Error) -> Self { CacheGlyphError::IoError(e) }
}

#[derive(Clone, Copy, Debug)]
pub struct CacheReadError;
impl Display for CacheReadError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use std::error::Error;
    write!(f, "{}", self.description())
  }
}

impl std::error::Error for CacheReadError {
  fn description(&self) -> &str { "The requested glyph was not in the cache" }
}

impl std::convert::From<CacheReadErr> for CacheReadError {
  fn from(_: CacheReadErr) -> Self { CacheReadError }
}

/// Enum for the gen_charset convenience function.
#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Charset {
  /// [a-z]
  Lowercase, 
  /// [A-Z]
  Uppercase, 
  /// [0-9]
  Numbers, 
  /// ISO/IEC 8859-1 Punctuation
  Punctuation,
}

/// Convenience function to generate commonly-used charsets in the form needed
/// for FontCache::cache_glyphs().
pub fn gen_charset(sets: &HashSet<Charset>) -> Vec<char> {
  let mut chars = Vec::new();
  for c in sets.iter() {
    match *c {
      Charset::Lowercase => {
        chars.extend_from_slice(&['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
                                'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
                                's', 't', 'u', 'v', 'w', 'x', 'y', 'z']);
      },
      Charset::Uppercase => {
        chars.extend_from_slice(&['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
                                'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
                                'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ]);
      },
      Charset::Numbers => {
        chars.extend_from_slice(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
      },
      Charset::Punctuation => {
        chars.extend_from_slice(&[' ', '!', '"', '#', '£', '$', '%', '&', '\'', '(',
                                ')', '*', '+', ',', '-', '.', '/', ':', ';',
                                '<', '=', '>', '?', '@', '[', '\\', ']', '^',
                                '_', '`', '{', '|', '}', '~', '¬']);
      }
    }
  }
  return chars;
}

/// A trait for a GPU font cache. Glyphs are loaded into the font cache,
/// which are stored on the GPU for fast access when rendering text.
pub trait FontCache : GlyphLookup { 
  /// A function to add the given chars to the cache. Duplicate chars will be
  /// ignored. Repeated calls to this function with the same file path and
  /// scale will be taken into account, and not re-cached.
  /// # Params
  /// * `file` - The file path of the ttf font to use.
  /// * `scale` - The scale of the font. This is the 'pt' you find in most
  ///             programs - 24pt is 24.0 for example.
  /// * `charset` - A slice of chars to render to the cache with this filename
  ///               and scale. Duplicate chars are ignored.
  /// # Errors
  /// Will return a CacheGlyph error if this function failed to add the glyphs to the cache.
  fn cache_glyphs<F: AsRef<Path>>(&mut self, file: F, scale: f32, charset: &[char]) 
    -> Result<FontHandle, CacheGlyphError>;
}

/// A trait which has methods for looking up UVs for a glyph given a font handle and a code point.
/// This is separate to FontCache to allow for more specialised objects to be used for lookup that
/// don't contain a GL context (allowing for send + sync).
pub trait GlyphLookup {
  /// A function to look up the texture coordinates of a given glyph.
  /// # Params
  /// * `font_handle` - The handle of the font this glyph was cached into with.
  /// * `code_point` - The code_point of the glyph to look up
  /// # Returns
  /// A result. When Ok, contains an option, containing the tex coords of the
  /// glyph requested. This can be null if the glyph isn't a glyph which needs
  /// to be rendered - i.e. the 'space' character.
  /// # Errors
  /// Will return a CacheReadError if the glyph was not cached.
  fn rect_for(&self, font_handle: FontHandle, code_point: char) 
    -> Result<Option<[f32; 4]>, CacheReadError>;

  /// Get a reference to the font (and scale x, y) attached to the given font
  /// handle.
  fn get_font_ref(&self, fh: FontHandle) -> Option<&(Font, (f32, f32))>;

  /// A function to get a glyph in the cache, given a font handle and a character.
  /// # Returns
  /// * Some(glyph) if the glyph was found.
  /// * None if the glyph has never been cached.
  /// # Notes
  /// This function returning Some is NOT a guarantee that the given glyph is
  /// currently store in the cache, and requesting a texture rect for the given
  /// glyph may still not return a value.
  fn get_glyph(&self, fh: FontHandle, c: char) -> Option<PositionedGlyph>;
}


/// A struct containing data to uniquely identify a font. Fonts are identified
/// by paths and sizes - so if you have 2 identical font files, but stored at
/// different paths, they will be stored separately in the cache. 
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FontSpec {
  path: PathBuf,
  /// The x scale of this font * 100. A font of size 24pt will have 24 * 100 * dpi x_scale
  /// and 24 * 100 * dpi y_scale. This is not stored as a floating point number
  /// because it needs to be the key in a map, and as such must implement Eq
  /// and Ord (or Hash, if using a hash map over a btreemap).
  x_scale: u32,
  /// The y scale of this font - see x_scale documentation for more details.
  y_scale: u32,
}
impl FontSpec {
  /// Create a new font spec. The x scale and y scale are 100 times the actual
  /// scale - for a font of size 24, use 2400 as the values for x and y scale.
  pub fn new<F: AsRef<Path>>(path: F, x_scale: u32, y_scale: u32) -> FontSpec {
    FontSpec {
      path: path.as_ref().to_path_buf(),
      x_scale: x_scale,
      y_scale: y_scale,
    }
  }
}

/// A font handle, to be owned by the end user and used to query for glyph
/// textures.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Debug)]
pub struct FontHandle(usize);
impl Deref for FontHandle {
  type Target = usize;
  fn deref(&self) -> &Self::Target { &self.0 }
}

