pub trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Deserializer<D, R> {
    fn deserialize<T: Deserialize<D, R>>(&mut self) -> R;

}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor<D, R> {
    fn visit(&mut self, state: &mut S) -> Option<R>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait SeqVisitor {

}

pub trait VisitorState<E> {
    fn syntax_error(&mut self) -> E;

    fn visit_int(&mut self) -> Result<int, E> {
    }

    fn visit_seq<Iter: FromIterator>(&mut self, ) -> Iter

}











///////////////////////////////////////////////////////////////////////////////

impl<D: VisitorState<E>, E> Deserialize<D, E> for int {
    #[inline]
    fn deserialize(state: &mut D) -> Result<int, E> {
        state.visit_int()
    }
}


impl<
    T: Deserialize<D, E>,
    D: VisitorState<E>,
    E
> Deserialize<D, E> for Vec<T> {
    #[inline]
    fn deserialize(state: &mut D) -> Result<int, E> {
        struct SeqVisitor {

        }

        impl SeqVisitor {
            fn visit_seq<V: Visitor<Self>>(len: uint, visitor: V) {
                let mut value = Vec::with_capacity(len);

                loop {
                    match visitor.visit(self, &mut value) {
                        Some(()) => { }
                        None => { break; }
                    }
                }

                value
            }

            fn visit_seq_elt<
                T: S
            >(&mut self, value: &mut Vec<T>) {
                let v = Deserialize
                value.push(
            }
        }

        let v = Vec::new();
        state.visit_seq()
    }
}
