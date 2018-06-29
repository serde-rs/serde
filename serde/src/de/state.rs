use lib::*;

/// An internally cow structure to store state extensions.
#[derive(Clone)]
pub struct State {
    #[cfg(feature = "state")]
    map: Option<Rc<Vec<(TypeId, Rc<Box<Any>>)>>>,
}

#[cfg(feature = "state")]
const EMPTY_STATE: State = State { map: None };
#[cfg(not(feature = "state"))]
const EMPTY_STATE: State = State {};

impl State {
    /// Creates an empty state map.
    #[inline]
    pub fn empty() -> &'static State {
        &EMPTY_STATE
    }

    /// Invokes a callback with the value.
    pub fn with<T: 'static, R, F: FnOnce(Option<&T>) -> R>(&self, f: F) -> R {
        #[cfg(feature = "state")] {
            if let Some(ref map) = self.map {
                for &(type_id, ref boxed_rc) in map.iter() {
                    if type_id == TypeId::of::<T>() {
                        return f((&***boxed_rc as &(Any + 'static)).downcast_ref());
                    }
                }
            }
        }
        f(None)
    }

    /// Inserts or replaces a type in the state map.
    pub fn set<T: 'static>(&mut self, val: T) {
        #[cfg(feature = "state")] {
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
        #[cfg(not(feature = "state"))] {
            let _val = val;
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("State").finish()
    }
}
