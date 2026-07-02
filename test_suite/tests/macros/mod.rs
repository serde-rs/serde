#![allow(unused_macro_rules)]

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
    ($hasher:ty; $($value:expr),+) => {{
        let mut set = HashSet::<_, $hasher>::default();
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
    ($hasher:ty; $($key:expr => $value:expr),+) => {{
        let mut map = HashMap::<_, _, $hasher>::default();
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
