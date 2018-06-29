use std::rc::Rc;
use std::any::{Any, TypeId};
use std::fmt;
use std::iter;

/// An internally cow structure to store state extensions.
#[derive(Clone)]
pub struct State {
    map: Option<Rc<Vec<(TypeId, Rc<Box<Any>>)>>>,
}

const EMPTY_STATE: State = State { map: None };

impl State {
    /// Creates an empty state map.
    #[inline]
    pub fn empty() -> &'static State {
        &EMPTY_STATE
    }

    /// Invokes a callback with the value.
    pub fn with<T: 'static, R, F: FnOnce(Option<&T>) -> R>(&self, f: F) -> R {
        if let Some(ref map) = self.map {
            for &(type_id, ref boxed_rc) in map.iter() {
                if type_id == TypeId::of::<T>() {
                    return f((&***boxed_rc as &(Any + 'static)).downcast_ref());
                }
            }
        }
        f(None)
    }

    /// Inserts or replaces a type in the state map.
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
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("State").finish()
    }
}
