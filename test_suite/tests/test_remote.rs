#![allow(clippy::redundant_field_names)]

use serde_derive::{Deserialize, Serialize};

mod remote {
    pub struct Unit;

    pub struct PrimitivePriv(u8);

    pub struct PrimitivePub(pub u8);

    pub struct NewtypePriv(Unit);

    pub struct NewtypePub(pub Unit);

    pub struct TuplePriv(u8, Unit);

    pub struct TuplePub(pub u8, pub Unit);

    pub struct StructPriv {
        a: u8,
        b: Unit,
    }

    pub struct StructPub {
        pub a: u8,
        pub b: Unit,
    }

    impl PrimitivePriv {
        pub fn new(a: u8) -> Self {
            PrimitivePriv(a)
        }

        pub fn get(&self) -> u8 {
            self.0
        }
    }

    impl NewtypePriv {
        pub fn new(a: Unit) -> Self {
            NewtypePriv(a)
        }

        pub fn get(&self) -> &Unit {
            &self.0
        }
    }

    impl TuplePriv {
        pub fn new(a: u8, b: Unit) -> Self {
            TuplePriv(a, b)
        }

        pub fn first(&self) -> u8 {
            self.0
        }

        pub fn second(&self) -> &Unit {
            &self.1
        }
    }

    impl StructPriv {
        pub fn new(a: u8, b: Unit) -> Self {
            StructPriv { a: a, b: b }
        }

        pub fn a(&self) -> u8 {
            self.a
        }

        pub fn b(&self) -> &Unit {
            &self.b
        }
    }

    pub struct StructGeneric<T> {
        pub value: T,
    }

    impl<T> StructGeneric<T> {
        #[allow(dead_code)]
        pub fn get_value(&self) -> &T {
            &self.value
        }
    }

    pub enum EnumGeneric<T> {
        Variant(T),
    }
}

#[derive(Serialize, Deserialize)]
struct Test {
    #[serde(with = "UnitDef")]
    unit: remote::Unit,

    #[serde(with = "PrimitivePrivDef")]
    primitive_priv: remote::PrimitivePriv,

    #[serde(with = "PrimitivePubDef")]
    primitive_pub: remote::PrimitivePub,

    #[serde(with = "NewtypePrivDef")]
    newtype_priv: remote::NewtypePriv,

    #[serde(with = "NewtypePubDef")]
    newtype_pub: remote::NewtypePub,

    #[serde(with = "TuplePrivDef")]
    tuple_priv: remote::TuplePriv,

    #[serde(with = "TuplePubDef")]
    tuple_pub: remote::TuplePub,

    #[serde(with = "StructPrivDef")]
    struct_priv: remote::StructPriv,

    #[serde(with = "StructPubDef")]
    struct_pub: remote::StructPub,

    #[serde(with = "StructConcrete")]
    struct_concrete: remote::StructGeneric<u8>,

    #[serde(with = "EnumConcrete")]
    enum_concrete: remote::EnumGeneric<u8>,

    #[serde(with = "ErrorKindDef")]
    io_error_kind: std::io::ErrorKind,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::Unit")]
struct UnitDef;

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::PrimitivePriv")]
struct PrimitivePrivDef(#[serde(getter = "remote::PrimitivePriv::get")] u8);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::PrimitivePub")]
struct PrimitivePubDef(u8);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::NewtypePriv")]
struct NewtypePrivDef(#[serde(getter = "remote::NewtypePriv::get", with = "UnitDef")] remote::Unit);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::NewtypePub")]
struct NewtypePubDef(#[serde(with = "UnitDef")] remote::Unit);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::TuplePriv")]
struct TuplePrivDef(
    #[serde(getter = "remote::TuplePriv::first")] u8,
    #[serde(getter = "remote::TuplePriv::second", with = "UnitDef")] remote::Unit,
);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::TuplePub")]
struct TuplePubDef(u8, #[serde(with = "UnitDef")] remote::Unit);

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::StructPriv")]
struct StructPrivDef {
    #[serde(getter = "remote::StructPriv::a")]
    a: u8,

    #[serde(getter = "remote::StructPriv::b")]
    #[serde(with = "UnitDef")]
    b: remote::Unit,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::StructPub")]
struct StructPubDef {
    a: u8,

    #[serde(with = "UnitDef")]
    b: remote::Unit,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::StructGeneric")]
struct StructGenericWithGetterDef<T> {
    #[serde(getter = "remote::StructGeneric::get_value")]
    value: T,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::StructGeneric<u8>")]
struct StructConcrete {
    value: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "remote::EnumGeneric<u8>")]
enum EnumConcrete {
    Variant(u8),
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "std::io::ErrorKind")]
#[non_exhaustive]
enum ErrorKindDef {
    NotFound,
    PermissionDenied,
    // ...
}

impl From<PrimitivePrivDef> for remote::PrimitivePriv {
    fn from(def: PrimitivePrivDef) -> Self {
        remote::PrimitivePriv::new(def.0)
    }
}

impl From<NewtypePrivDef> for remote::NewtypePriv {
    fn from(def: NewtypePrivDef) -> Self {
        remote::NewtypePriv::new(def.0)
    }
}

impl From<TuplePrivDef> for remote::TuplePriv {
    fn from(def: TuplePrivDef) -> Self {
        remote::TuplePriv::new(def.0, def.1)
    }
}

impl From<StructPrivDef> for remote::StructPriv {
    fn from(def: StructPrivDef) -> Self {
        remote::StructPriv::new(def.a, def.b)
    }
}

impl<T> From<StructGenericWithGetterDef<T>> for remote::StructGeneric<T> {
    fn from(def: StructGenericWithGetterDef<T>) -> Self {
        remote::StructGeneric { value: def.value }
    }
}
