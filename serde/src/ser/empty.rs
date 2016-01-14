//! Helper trait for code generation purposes
//! Most collections utilize `is_empty` method to check for emptiness
//! But some container types, like `Option<T>`, have that method named
//! differently, f.e. `is_none`. Different naming enforces two different
//! attributes to check for emptiness. And this trait provides naming alias
//! whenever you like

/// Empty trait itself, very simple
pub trait Empty {
	/// Common alias method for emptiness check, like with most collection types
	fn is_empty(&self) -> bool;
}

impl<T> Empty for Option<T> {
	fn is_empty(&self) -> bool {
		self.is_none()
	}
}
