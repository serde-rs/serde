use lib::*;

/// A copy on write structure for deserializer state.
///
/// Deserializers can hold arbitrary state that the `Deserializer` trait
/// can access.  From the outside state can only be read not modified
/// (not counting interior mutability).  Any static type can be added
/// to the state.
///
/// This for instance can be used to notify types about location
/// information or associated metadata.  If the `state` feature is
/// disabled in serde this becomes a zero sized type that does not
/// do anything instead.
///
/// Internally the structure is a reasonably efficient copy on write
/// structure.  It can be cheaply cloned which effectively just bumps
/// some refcounts.  Internally it is implemented as a vector.
///
/// This requires Rust 1.22 or later and the `state` feature to be
/// enabled.  Otherwise the state type is read-only and does not
/// provide the `set` and `remove` methods.
#[derive(Clone)]
pub struct State {
    #[cfg(all(feature = "state", de_state))]
    map: Option<Rc<Vec<(TypeId, Rc<Box<Any>>)>>>,
}

impl State {
    /// Returns the static reference to the empty state.
    ///
    /// The state is normally non `Send` but the read only empty state
    /// can be safely accessed from multiple threads.  To modify the
    /// state it needs to be cloned first.
    ///
    /// ```
    /// # use serde::de::State;
    /// struct MyInfo(i32);
    /// let mut state = State::empty().clone();
    /// state.set(MyInfo(42));
    /// ```
    #[inline]
    pub fn empty() -> &'static State {
        // we could use `const EMPTY_STATE: State` here for newer rust
        // versions which would avoid the unsafe.  The end result is
        // about the same though.
        #[cfg(all(feature = "state", de_state))] {
            const EMPTY_STATE: State = State {
                map: None
            };
            &EMPTY_STATE
        }
        #[cfg(not(all(feature = "state", de_state)))] {
            static mut EMPTY_STATE: State = State {};
            unsafe { &EMPTY_STATE }
        }
    }

    /// Looks up an item.
    ///
    /// This function is always available even if the state feature is
    /// disabled.  In that case the state just always returns `None`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        #[cfg(all(feature = "state", de_state))] {
            if let Some(ref map) = self.map {
                for &(type_id, ref boxed_rc) in map.iter() {
                    if type_id == TypeId::of::<T>() {
                        return (&***boxed_rc as &(Any + 'static)).downcast_ref();
                    }
                }
            }
        }
        None
    }

    /// Inserts or replaces a type in the state map.
    #[cfg(all(feature = "state", de_state))]
    pub fn set<T: 'static>(&mut self, val: T) {
        self.map = Some(Rc::new(self.map
            .as_ref()
            .map(|x| &x[..])
            .unwrap_or(&[][..])
            .iter()
            .filter_map(|&(type_id, ref boxed_rc)| {
                if type_id != TypeId::of::<T>() {
                    Some((type_id, boxed_rc.clone()))
                } else {
                    None
                }
            })
            .chain(iter::once((TypeId::of::<T>(), Rc::new(Box::new(val) as Box<Any>))))
            .collect()));
    }

    /// Removes a type from the state map.
    #[cfg(all(feature = "state", de_state))]
    pub fn remove<T: 'static>(&mut self) {
        let new_map = {
            let mut iter = self.map
                .as_ref()
                .map(|x| &x[..])
                .unwrap_or(&[][..])
                .iter()
                .filter_map(|&(type_id, ref boxed_rc)| {
                    if type_id != TypeId::of::<T>() {
                        Some((type_id, boxed_rc.clone()))
                    } else {
                        None
                    }
                })
                .peekable();
            if iter.peek().is_some() {
                Some(iter.collect())
            } else {
                None
            }
        };
        self.map = new_map.map(Rc::new);
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("State").finish()
    }
}
