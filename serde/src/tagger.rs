/// Provides the correct tag for a given format.
pub trait Tagger {
    /// Returns a `u8` tag if available.
    fn u8_tag(&self, _format: &'static str) -> Option<u8> { None }
    
    /// Returns a `u16` tag if available.
    fn u16_tag(&self, _format: &'static str) -> Option<u16> { None }
    
    /// Returns a `u32` tag if available.
    fn u32_tag(&self, _format: &'static str) -> Option<u32> { None }
    
    /// Returns a `u64` tag if available.
    fn u64_tag(&self, _format: &'static str) -> Option<u64> { None }
    
    /// Returns an `i8` tag if available.
    fn i8_tag(&self, _format: &'static str) -> Option<i8> { None }
    
    /// Returns an `i16` tag if available.
    fn i16_tag(&self, _format: &'static str) -> Option<i16> { None }
    
    /// Returns an `i32` tag if available.
    fn i32_tag(&self, _format: &'static str) -> Option<i32> { None }
    
    /// Returns an `i64` tag if available.
    fn i64_tag(&self, _format: &'static str) -> Option<i64> { None }
    
    /// Provides a string tag if available.
    fn string_tag(&self, _format: &'static str) -> Option<&str> { None }
    
    /// Provides a binary tag if available.
    fn bytes_tag(&self, _format: &'static str) -> Option<&[u8]> { None }
}
