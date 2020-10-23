use serde::{Deserialize, Serialize};

#[test]
fn test_self() {
    macro_rules! mac {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }

    pub trait Trait {
        type Assoc;
    }

    #[derive(Deserialize, Serialize)]
    pub struct Generics<T: Trait<Assoc = Self>>
    where
        Self: Trait<Assoc = Self>,
        <Self as Trait>::Assoc: Sized,
        mac!(Self): Trait<Assoc = mac!(Self)>,
    {
        _f: T,
    }

    impl<T: Trait<Assoc = Self>> Trait for Generics<T> {
        type Assoc = Self;
    }

    #[derive(Deserialize, Serialize)]
    pub struct Struct {
        _f1: Box<Self>,
        _f2: Box<<Self as Trait>::Assoc>,
        _f3: Box<mac!(Self)>,
        _f4: [(); Self::ASSOC],
        _f5: [(); Self::assoc()],
        _f6: [(); mac!(Self::assoc())],
    }

    impl Struct {
        const ASSOC: usize = 1;
        const fn assoc() -> usize {
            0
        }
    }

    impl Trait for Struct {
        type Assoc = Self;
    }

    #[derive(Deserialize, Serialize)]
    struct Tuple(
        Box<Self>,
        Box<<Self as Trait>::Assoc>,
        Box<mac!(Self)>,
        [(); Self::ASSOC],
        [(); Self::assoc()],
        [(); mac!(Self::assoc())],
    );

    impl Tuple {
        const ASSOC: usize = 1;
        const fn assoc() -> usize {
            0
        }
    }

    impl Trait for Tuple {
        type Assoc = Self;
    }

    #[derive(Deserialize, Serialize)]
    enum Enum {
        Struct {
            _f1: Box<Self>,
            _f2: Box<<Self as Trait>::Assoc>,
            _f3: Box<mac!(Self)>,
            _f4: [(); Self::ASSOC],
            _f5: [(); Self::assoc()],
            _f6: [(); mac!(Self::assoc())],
        },
        Tuple(
            Box<Self>,
            Box<<Self as Trait>::Assoc>,
            Box<mac!(Self)>,
            [(); Self::ASSOC],
            [(); Self::assoc()],
            [(); mac!(Self::assoc())],
        ),
    }

    impl Enum {
        const ASSOC: usize = 1;
        const fn assoc() -> usize {
            0
        }
    }

    impl Trait for Enum {
        type Assoc = Self;
    }
}
