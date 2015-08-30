#[macro_export]
macro_rules! declare_ser_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            #[test]
            fn $name() {
                $(
                    ::token::assert_ser_tokens(&$value, $tokens);
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
    }
}
