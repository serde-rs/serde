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

impl<'a, T: Empty> Empty for &'a T {
    fn is_empty(&self) -> bool {
        Empty::is_empty(*self)
    }
}

impl<T> Empty for [T] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl Empty for str {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

use std;
use std::collections as c;

impl<K: std::hash::Hash + Eq, V> Empty for c::HashMap<K, V> {
    fn is_empty(&self) -> bool {
        c::HashMap::is_empty(self)
    }
}

impl<T: std::hash::Hash + Eq> Empty for c::HashSet<T> {
    fn is_empty(&self) -> bool {
        c::HashSet::is_empty(self)
    }
}

impl<K, V> Empty for c::BTreeMap<K, V> {
    fn is_empty(&self) -> bool {
        c::BTreeMap::is_empty(self)
    }
}

impl<T: Ord> Empty for c::BTreeSet<T> {
    fn is_empty(&self) -> bool {
        c::BTreeSet::is_empty(self)
    }
}

impl<T: Ord> Empty for c::binary_heap::BinaryHeap<T> {
    fn is_empty(&self) -> bool {
        c::binary_heap::BinaryHeap::is_empty(self)
    }
}

impl<T> Empty for c::linked_list::LinkedList<T> {
    fn is_empty(&self) -> bool {
        c::linked_list::LinkedList::is_empty(self)
    }
}

impl Empty for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
}

impl<T> Empty for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

impl<T> Empty for c::vec_deque::VecDeque<T> {
    fn is_empty(&self) -> bool {
        c::VecDeque::is_empty(self)
    }
}
