#[macro_export]
macro_rules! declare_ser_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            #[test]
            fn $name() {
                $(
                    assert_ser_tokens(&$value, $tokens);
                )+
            }
        )+
    }
}

#[macro_export]
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
