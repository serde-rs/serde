macro_rules! btreeset {
    () => {
        BTreeSet::new()
    };
    ($($value:expr),+) => {
        {
            let mut set = BTreeSet::new();
            $(set.insert($value);)+
            set
        }
    }
}

macro_rules! btreemap {
    () => {
        BTreeMap::new()
    };
    ($($key:expr => $value:expr),+) => {
        {
            let mut map = BTreeMap::new();
            $(map.insert($key, $value);)+
            map
        }
    }
}

macro_rules! hashset {
    () => {
        HashSet::new()
    };
    ($($value:expr),+) => {
        {
            let mut set = HashSet::new();
            $(set.insert($value);)+
            set
        }
    };
    ($hasher:ident @ $($value:expr),+) => {
        {
            use std::hash::BuildHasherDefault;
            let mut set = HashSet::with_hasher(BuildHasherDefault::<$hasher>::default());
            $(set.insert($value);)+
            set
        }
    }
}

macro_rules! hashmap {
    () => {
        HashMap::new()
    };
    ($($key:expr => $value:expr),+) => {
        {
            let mut map = HashMap::new();
            $(map.insert($key, $value);)+
            map
        }
    };
    ($hasher:ident @ $($key:expr => $value:expr),+) => {
        {
            use std::hash::BuildHasherDefault;
            let mut map = HashMap::with_hasher(BuildHasherDefault::<$hasher>::default());
            $(map.insert($key, $value);)+
            map
        }
    }
}

macro_rules! seq_impl {
    (seq $first:expr,) => {
        seq_impl!(seq $first)
    };
    ($first:expr,) => {
        seq_impl!($first)
    };
    (seq $first:expr) => {
        $first.into_iter()
    };
    ($first:expr) => {
        Some($first).into_iter()
    };
    (seq $first:expr , $( $elem: tt)*) => {
        $first.into_iter().chain(seq!( $($elem)* ))
    };
    ($first:expr , $($elem: tt)*) => {
        Some($first).into_iter().chain(seq!( $($elem)* ))
    }
}
macro_rules! seq {
    ($($tt: tt)*) => {
        seq_impl!($($tt)*).collect::<Vec<_>>()
    };
}
