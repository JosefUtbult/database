#[cfg(test)]
pub(crate) mod test_types {

    use crate::{
        AbsFolderConstraints, FlatFolderConstraints, ToKey,
        database_traits::{
            AbsFieldConstraints, AbsKeyConstraints, AllVariants, FlatFieldConstraints,
            FlatKeyConstraints, UsizeConstraints, VariantCount,
        },
    };

    pub(crate) struct MyInnerInnerData {
        pub(crate) param6: u8,
    }

    impl MyInnerInnerData {
        pub(crate) const fn new() -> Self {
            Self {
                param6: 0
            }
        }
    }

    pub(crate) struct MyInnerData {
        pub(crate) param4: u8,
        pub(crate) param5: bool,
        pub(crate) inner3: MyInnerInnerData,
        pub(crate) inner4: MyInnerInnerData,
    }

    impl MyInnerData {
        pub(crate) const fn new() -> Self {
            Self {
                param4: 0,
                param5: false,
                inner3: MyInnerInnerData::new(),
                inner4: MyInnerInnerData::new()
            }
        }
    }

    pub(crate) struct MyLayerData {
        pub(crate) param1: u8,
        pub(crate) param2: bool,
        pub(crate) param3: u8,
        pub(crate) inner1: MyInnerData,
        pub(crate) inner2: MyInnerData,
    }

    impl MyLayerData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0,
                inner1: MyInnerData::new(),
                inner2: MyInnerData::new(),
            }
        }
    }

    pub(crate) struct MyFlatData {
        pub(crate) param1: u8,
        pub(crate) param2: bool,
        pub(crate) param3: u8,
        pub(crate) param4: u8,
        pub(crate) param5: bool,
        pub(crate) param6: u8,
    }

    impl MyFlatData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0,
                param4: 0,
                param5: false,
                param6: 0
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerInnerDataKeys {
        Param6,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerInnerDataFields {
        Param6(u8)
    }

    impl ToKey<MyInnerInnerDataKeys> for MyInnerInnerDataFields {
        fn to_key(&self) -> MyInnerInnerDataKeys {
            match self {
                MyInnerInnerDataFields::Param6(_) => MyInnerInnerDataKeys::Param6
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataKeys {
        Param4,
        Param5,
        Inner3(MyInnerInnerDataKeys),
        Inner4(MyInnerInnerDataKeys),
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataFields {
        Param4(u8),
        Param5(bool),
        Inner3(MyInnerInnerDataFields),
        Inner4(MyInnerInnerDataFields),
    }

    impl ToKey<MyInnerDataKeys> for MyInnerDataFields {
        fn to_key(&self) -> MyInnerDataKeys {
            match self {
                MyInnerDataFields::Param4(_) => MyInnerDataKeys::Param4,
                MyInnerDataFields::Param5(_) => MyInnerDataKeys::Param5,
                MyInnerDataFields::Inner3(field) => MyInnerDataKeys::Inner3(field.to_key()),
                MyInnerDataFields::Inner4(field) => MyInnerDataKeys::Inner4(field.to_key()),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataFolders {
        Inner3,
        Inner4
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataAbsKeys {
        Param1,
        Param2,
        Param3,
        Inner1(MyInnerDataKeys),
        Inner2(MyInnerDataKeys),
    }
    impl AbsKeyConstraints for MyDataAbsKeys {}

    pub(crate) const MY_DATA_ABS_VARIANT_COUNT: usize = 11;
    pub(crate) const MY_DATA_ABS_KEYS_ALL_VARIANTS: [MyDataAbsKeys; MY_DATA_ABS_VARIANT_COUNT] = [
        MyDataAbsKeys::Param1,
        MyDataAbsKeys::Param2,
        MyDataAbsKeys::Param3,
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param5),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6)),
    ];

    impl VariantCount for MyDataAbsKeys {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    impl AllVariants for MyDataAbsKeys {
        const ALL_VARIANTS: &[Self] = &MY_DATA_ABS_KEYS_ALL_VARIANTS;
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataAbsFolders {
        Inner1,
        InInner1(MyInnerDataFolders),
        Inner2,
        InInner2(MyInnerDataFolders),
    }
    impl VariantCount for MyDataAbsFolders {
        const COUNT: usize = 2;
    }
    impl AbsFolderConstraints for MyDataAbsFolders {}

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataAbsFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Inner1(MyInnerDataFields),
        Inner2(MyInnerDataFields),
    }
    impl AbsFieldConstraints<MyDataAbsKeys> for MyDataAbsFields {}

    impl VariantCount for MyDataAbsFields {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    impl ToKey<MyDataAbsKeys> for MyDataAbsFields {
        fn to_key(&self) -> MyDataAbsKeys {
            match self {
                MyDataAbsFields::Param1(_) => MyDataAbsKeys::Param1,
                MyDataAbsFields::Param2(_) => MyDataAbsKeys::Param2,
                MyDataAbsFields::Param3(_) => MyDataAbsKeys::Param3,
                MyDataAbsFields::Inner1(inner) => MyDataAbsKeys::Inner1(inner.to_key()),
                MyDataAbsFields::Inner2(inner) => MyDataAbsKeys::Inner2(inner.to_key()),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataFlatKeys {
        Param1,
        Param2,
        Param3,
        Param4,
        Param5,
        Param6
    }
    impl AbsKeyConstraints for MyDataFlatKeys {}
    impl FlatKeyConstraints<MyDataAbsKeys> for MyDataFlatKeys {}
    impl FlatKeyConstraints<MyDataFlatKeys> for MyDataFlatKeys {}
    impl UsizeConstraints<MyDataFlatKeys> for usize {}

    impl From<MyDataFlatKeys> for usize {
        fn from(value: MyDataFlatKeys) -> Self {
            match value {
                MyDataFlatKeys::Param1 => 0,
                MyDataFlatKeys::Param2 => 1,
                MyDataFlatKeys::Param3 => 2,
                MyDataFlatKeys::Param4 => 3,
                MyDataFlatKeys::Param5 => 4,
                MyDataFlatKeys::Param6 => 4,
            }
        }
    }

    pub(crate) const MY_DATA_FLAT_VARIANT_COUNT: usize = 6;
    pub(crate) const MY_DATA_FLAT_KEYS_ALL_VARIANTS: [MyDataFlatKeys; MY_DATA_FLAT_VARIANT_COUNT] = [
        MyDataFlatKeys::Param1,
        MyDataFlatKeys::Param2,
        MyDataFlatKeys::Param3,
        MyDataFlatKeys::Param4,
        MyDataFlatKeys::Param5,
        MyDataFlatKeys::Param6,
    ];

    impl VariantCount for MyDataFlatKeys {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl AllVariants for MyDataFlatKeys {
        const ALL_VARIANTS: &[Self] = &MY_DATA_FLAT_KEYS_ALL_VARIANTS;
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataFlatFolders {
        Inner1,
        Inner2,
        Inner3,
        Inner4
    }
    impl VariantCount for MyDataFlatFolders {
        const COUNT: usize = 4;
    }
    impl AbsFolderConstraints for MyDataFlatFolders {}
    impl FlatFolderConstraints<MyDataAbsFolders> for MyDataFlatFolders {}
    impl FlatFolderConstraints<MyDataFlatFolders> for MyDataFlatFolders {}

    impl From<MyDataAbsFolders> for MyDataFlatFolders {
        fn from(value: MyDataAbsFolders) -> Self {
            match value {
                MyDataAbsFolders::Inner1 => MyDataFlatFolders::Inner1,
                MyDataAbsFolders::Inner2 => MyDataFlatFolders::Inner2,
                MyDataAbsFolders::Inner3 => MyDataFlatFolders::Inner3,
                MyDataAbsFolders::Inner4 => MyDataFlatFolders::Inner4,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataFlatFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Param4(u8),
        Param5(bool),
        Param6(u8),
    }
    impl FlatFieldConstraints<MyDataAbsFields, MyDataFlatKeys> for MyDataFlatFields {}
    impl FlatFieldConstraints<MyDataFlatFields, MyDataFlatKeys> for MyDataFlatFields {}
    impl AbsFieldConstraints<MyDataFlatKeys> for MyDataFlatFields {}

    impl VariantCount for MyDataFlatFields {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl ToKey<MyDataFlatKeys> for MyDataFlatFields {
        fn to_key(&self) -> MyDataFlatKeys {
            match self {
                MyDataFlatFields::Param1(_) => MyDataFlatKeys::Param1,
                MyDataFlatFields::Param2(_) => MyDataFlatKeys::Param2,
                MyDataFlatFields::Param3(_) => MyDataFlatKeys::Param3,
                MyDataFlatFields::Param4(_) => MyDataFlatKeys::Param4,
                MyDataFlatFields::Param5(_) => MyDataFlatKeys::Param5,
                MyDataFlatFields::Param6(_) => MyDataFlatKeys::Param6,
            }
        }
    }

    impl From<MyDataAbsKeys> for MyDataFlatKeys {
        fn from(value: MyDataAbsKeys) -> Self {
            match value {
                MyDataAbsKeys::Param1 => MyDataFlatKeys::Param1,
                MyDataAbsKeys::Param2 => MyDataFlatKeys::Param2,
                MyDataAbsKeys::Param3 => MyDataFlatKeys::Param3,
                MyDataAbsKeys::Inner1(inner) | MyDataAbsKeys::Inner2(inner) => match inner {
                    MyInnerDataKeys::Param4 => MyDataFlatKeys::Param4,
                    MyInnerDataKeys::Param5 => MyDataFlatKeys::Param5,
                    MyInnerDataKeys::Inner3(inner) | MyInnerDataKeys::Inner4(inner) => match inner {
                        MyInnerInnerDataKeys::Param6 => MyDataFlatKeys::Param6,
                    }
                },
            }
        }
    }

    impl From<MyDataAbsFields> for MyDataFlatFields {
        fn from(value: MyDataAbsFields) -> Self {
            match value {
                MyDataAbsFields::Param1(value) => MyDataFlatFields::Param1(value),
                MyDataAbsFields::Param2(value) => MyDataFlatFields::Param2(value),
                MyDataAbsFields::Param3(value) => MyDataFlatFields::Param3(value),
                MyDataAbsFields::Inner1(inner) | MyDataAbsFields::Inner2(inner) => match inner {
                    MyInnerDataFields::Param4(value) => MyDataFlatFields::Param4(value),
                    MyInnerDataFields::Param5(value) => MyDataFlatFields::Param5(value),
                    MyInnerDataFields::Inner3(inner) | MyInnerDataFields::Inner4(inner) => match inner {
                        MyInnerInnerDataFields::Param6(value) => MyDataFlatFields::Param6(value),
                    }
                },
            }
        }
    }
}
