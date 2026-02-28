use database_macro::build_database;

use database::{
    AbsFieldConstraints, AbsKeyConstraints, AllVariants, DataFieldAccessor, FlatFieldConstraints,
    FlatKeyConstraints, FocusHandler, LayerDatabase, ToKey, VariantCount,
};

build_database!(
    struct C {
        my_field: u8,
    },
    struct B {
        c6: C,
        c7: C,
    },
    struct A {
        b4: B,
        c5: C,
    },
    struct Root {
        c1: C,
        a2: A,
        a3: A,
        b8: B,
    }
);

fn main() {}
