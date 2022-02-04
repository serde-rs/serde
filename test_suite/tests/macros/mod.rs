use serde_test::Token;
use std::iter;

macro_rules! btreeset {
    () => {
        BTreeSet::new()
    };
    ($($value:expr),+) => {{
        let mut set = BTreeSet::new();
        $(set.insert($value);)+
        set
    }};
}

macro_rules! btreemap {
    () => {
        BTreeMap::new()
    };
    ($($key:expr => $value:expr),+) => {{
        let mut map = BTreeMap::new();
        $(map.insert($key, $value);)+
        map
    }};
}

macro_rules! hashset {
    () => {
        HashSet::new()
    };
    ($($value:expr),+) => {{
        let mut set = HashSet::new();
        $(set.insert($value);)+
        set
    }};
    ($hasher:ident @ $($value:expr),+) => {{
        use std::hash::BuildHasherDefault;
        let mut set = HashSet::with_hasher(BuildHasherDefault::<$hasher>::default());
        $(set.insert($value);)+
        set
    }};
}

macro_rules! hashmap {
    () => {
        HashMap::new()
    };
    ($($key:expr => $value:expr),+) => {{
        let mut map = HashMap::new();
        $(map.insert($key, $value);)+
        map
    }};
    ($hasher:ident @ $($key:expr => $value:expr),+) => {{
        use std::hash::BuildHasherDefault;
        let mut map = HashMap::with_hasher(BuildHasherDefault::<$hasher>::default());
        $(map.insert($key, $value);)+
        map
    }};
}

pub trait SingleTokenIntoIterator {
    fn into_iter(self) -> iter::Once<Token>;
}

impl SingleTokenIntoIterator for Token {
    fn into_iter(self) -> iter::Once<Token> {
        iter::once(self)
    }
}

macro_rules! seq {
    ($($elem:expr),* $(,)?) => {{
        use crate::macros::SingleTokenIntoIterator;
        let mut vec = Vec::new();
        $(<Vec<Token> as Extend<Token>>::extend(&mut vec, $elem.into_iter());)*
        vec
    }};
}
