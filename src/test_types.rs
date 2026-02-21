#[cfg(test)]
pub(crate) mod test_types {

    use crate::database_traits::{
        AbsFieldConstraints, AbsKeyConstraints, AllVariants, FlatFieldConstraints,
        FlatKeyConstraints, UsizeConstraints, VariantCount,
    };
    use database_macro::build_database;

    pub(crate) struct MyInnerData {
        pub(crate) param4: u8,
        pub(crate) param5: bool,
    }

    impl MyInnerData {
        pub(crate) const fn new() -> Self {
            Self {
                param4: 0,
                param5: false,
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
    }

    impl MyFlatData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0,
                param4: 0,
                param5: false,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyInnerDataKeys {
        Param4,
        Param5,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyInnerDataFields {
        Param4(u8),
        Param5(bool),
    }

    impl From<MyInnerDataFields> for MyInnerDataKeys {
        fn from(value: MyInnerDataFields) -> Self {
            match value {
                MyInnerDataFields::Param4(_) => MyInnerDataKeys::Param4,
                MyInnerDataFields::Param5(_) => MyInnerDataKeys::Param5,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataAbsKeys {
        Param1,
        Param2,
        Param3,
        Inner1(MyInnerDataKeys),
        Inner2(MyInnerDataKeys),
    }
    impl AbsKeyConstraints<MyDataAbsFields> for MyDataAbsKeys {}

    enum MyDataWildcardKeys {
        Param1,
        Param2,
        Param3,
        Inner1(Option<MyInnerDataKeys>),
        Inner2(Option<MyInnerDataKeys>),
    }

    pub(crate) const MY_DATA_ABS_VARIANT_COUNT: usize = 7;
    pub(crate) const MY_DATA_ABS_KEYS_ALL_VARIANTS: [MyDataAbsKeys; MY_DATA_ABS_VARIANT_COUNT] = [
        MyDataAbsKeys::Param1,
        MyDataAbsKeys::Param2,
        MyDataAbsKeys::Param3,
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param5),
    ];

    impl VariantCount for MyDataAbsKeys {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    impl AllVariants for MyDataAbsKeys {
        const ALL_VARIANTS: &[Self] = &MY_DATA_ABS_KEYS_ALL_VARIANTS;
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataAbsFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Inner1(MyInnerDataFields),
        Inner2(MyInnerDataFields),
    }
    impl AbsFieldConstraints for MyDataAbsFields {}

    impl VariantCount for MyDataAbsFields {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataFlatKeys {
        Param1,
        Param2,
        Param3,
        Param4,
        Param5,
    }
    impl FlatKeyConstraints<MyDataAbsKeys, MyDataFlatFields> for MyDataFlatKeys {}
    impl FlatKeyConstraints<MyDataFlatKeys, MyDataFlatFields> for MyDataFlatKeys {}
    impl AbsKeyConstraints<MyDataFlatFields> for MyDataFlatKeys {}
    impl UsizeConstraints<MyDataFlatKeys> for usize {}

    impl From<MyDataFlatKeys> for usize {
        fn from(value: MyDataFlatKeys) -> Self {
            match value {
                MyDataFlatKeys::Param1 => 0,
                MyDataFlatKeys::Param2 => 1,
                MyDataFlatKeys::Param3 => 2,
                MyDataFlatKeys::Param4 => 3,
                MyDataFlatKeys::Param5 => 4,
            }
        }
    }

    pub(crate) const MY_DATA_FLAT_VARIANT_COUNT: usize = 5;
    pub(crate) const MY_DATA_FLAT_KEYS_ALL_VARIANTS: [MyDataFlatKeys; MY_DATA_FLAT_VARIANT_COUNT] = [
        MyDataFlatKeys::Param1,
        MyDataFlatKeys::Param2,
        MyDataFlatKeys::Param3,
        MyDataFlatKeys::Param4,
        MyDataFlatKeys::Param5,
    ];

    impl VariantCount for MyDataFlatKeys {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl AllVariants for MyDataFlatKeys {
        const ALL_VARIANTS: &[Self] = &MY_DATA_FLAT_KEYS_ALL_VARIANTS;
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataFlatFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Param4(u8),
        Param5(bool),
    }
    impl FlatFieldConstraints<MyDataAbsFields> for MyDataFlatFields {}
    impl FlatFieldConstraints<MyDataFlatFields> for MyDataFlatFields {}
    impl AbsFieldConstraints for MyDataFlatFields {}

    impl VariantCount for MyDataFlatFields {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl From<MyDataAbsFields> for MyDataAbsKeys {
        fn from(value: MyDataAbsFields) -> Self {
            match value {
                MyDataAbsFields::Param1(_) => MyDataAbsKeys::Param1,
                MyDataAbsFields::Param2(_) => MyDataAbsKeys::Param2,
                MyDataAbsFields::Param3(_) => MyDataAbsKeys::Param3,
                MyDataAbsFields::Inner1(inner1) => MyDataAbsKeys::Inner1(inner1.into()),
                MyDataAbsFields::Inner2(inner2) => MyDataAbsKeys::Inner2(inner2.into()),
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
                },
            }
        }
    }

    impl From<MyDataFlatFields> for MyDataFlatKeys {
        fn from(value: MyDataFlatFields) -> Self {
            match value {
                MyDataFlatFields::Param1(_) => MyDataFlatKeys::Param1,
                MyDataFlatFields::Param2(_) => MyDataFlatKeys::Param2,
                MyDataFlatFields::Param3(_) => MyDataFlatKeys::Param3,
                MyDataFlatFields::Param4(_) => MyDataFlatKeys::Param4,
                MyDataFlatFields::Param5(_) => MyDataFlatKeys::Param5,
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
                },
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum MyDataError {
        Invalid,
    }

    impl TryFrom<MyDataFlatFields> for u8 {
        type Error = MyDataError;

        fn try_from(value: MyDataFlatFields) -> Result<Self, Self::Error> {
            match value {
                MyDataFlatFields::Param1(value) => Ok(value),
                MyDataFlatFields::Param3(value) => Ok(value),
                MyDataFlatFields::Param4(value) => Ok(value),
                _ => Err(MyDataError::Invalid),
            }
        }
    }

    impl TryFrom<MyDataFlatFields> for bool {
        type Error = MyDataError;

        fn try_from(value: MyDataFlatFields) -> Result<Self, Self::Error> {
            match value {
                MyDataFlatFields::Param2(value) => Ok(value),
                MyDataFlatFields::Param5(value) => Ok(value),
                _ => Err(MyDataError::Invalid),
            }
        }
    }

}
