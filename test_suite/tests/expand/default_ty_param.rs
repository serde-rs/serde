use serde::{Deserialize, Serialize};

trait AssociatedType {
    type X;
}

impl AssociatedType for i32 {
    type X = i32;
}

#[derive(Serialize, Deserialize)]
struct DefaultTyParam<T: AssociatedType<X = i32> = i32> {
    phantom: PhantomData<T>,
}
