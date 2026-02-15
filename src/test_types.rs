#[cfg(test)]
pub(crate) mod test_types {
    use crate::{DataFieldAccessor, FocusHandler};

    pub(crate) struct MyInnerData {
        param4: u8,
        param5: bool,
    }

    impl MyInnerData {
        pub(crate) const fn new() -> Self {
            Self {
                param4: 0,
                param5: false,
            }
        }
    }

    pub(crate) struct MyData {
        param1: u8,
        param2: bool,
        param3: u8,
        inner1: MyInnerData,
    }

    impl MyData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0,
                inner1: MyInnerData::new(),
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
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataAbsFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Inner1(MyInnerDataFields),
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

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub(crate) enum MyDataFlatFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Param4(u8),
        Param5(bool),
    }

    impl From<MyDataAbsFields> for MyDataAbsKeys {
        fn from(value: MyDataAbsFields) -> Self {
            match value {
                MyDataAbsFields::Param1(_) => MyDataAbsKeys::Param1,
                MyDataAbsFields::Param2(_) => MyDataAbsKeys::Param2,
                MyDataAbsFields::Param3(_) => MyDataAbsKeys::Param3,
                MyDataAbsFields::Inner1(inner1) => MyDataAbsKeys::Inner1(inner1.into()),
            }
        }
    }

    impl From<MyDataAbsKeys> for MyDataFlatKeys {
        fn from(value: MyDataAbsKeys) -> Self {
            match value {
                MyDataAbsKeys::Param1 => MyDataFlatKeys::Param1,
                MyDataAbsKeys::Param2 => MyDataFlatKeys::Param2,
                MyDataAbsKeys::Param3 => MyDataFlatKeys::Param3,
                MyDataAbsKeys::Inner1(inner1) => match inner1 {
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
                MyDataAbsFields::Inner1(my_inner_data_flat_keys) => match my_inner_data_flat_keys {
                    MyInnerDataFields::Param4(value) => MyDataFlatFields::Param4(value),
                    MyInnerDataFields::Param5(value) => MyDataFlatFields::Param5(value),
                },
            }
        }
    }

    pub(crate) const MY_DATA_PARAMETER_FLAT_COUNT: usize = 5;

    pub(crate) const ALL_MY_FLAT_KEYS: [MyDataFlatKeys; MY_DATA_PARAMETER_FLAT_COUNT] = [
        MyDataFlatKeys::Param1,
        MyDataFlatKeys::Param2,
        MyDataFlatKeys::Param3,
        MyDataFlatKeys::Param4,
        MyDataFlatKeys::Param5,
    ];

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

    impl DataFieldAccessor<MyInnerDataKeys, MyInnerDataFields> for MyInnerData {
        fn get(&self, key: MyInnerDataKeys) -> MyInnerDataFields {
            match key {
                MyInnerDataKeys::Param4 => MyInnerDataFields::Param4(self.param4),
                MyInnerDataKeys::Param5 => MyInnerDataFields::Param5(self.param5),
            }
        }

        fn set(&mut self, field: MyInnerDataFields) {
            match field {
                MyInnerDataFields::Param4(value) => self.param4 = value,
                MyInnerDataFields::Param5(value) => self.param5 = value,
            }
        }
    }

    impl DataFieldAccessor<MyDataAbsKeys, MyDataAbsFields> for MyData {
        fn get(&self, key: MyDataAbsKeys) -> MyDataAbsFields {
            match key {
                MyDataAbsKeys::Param1 => MyDataAbsFields::Param1(self.param1),
                MyDataAbsKeys::Param2 => MyDataAbsFields::Param2(self.param2),
                MyDataAbsKeys::Param3 => MyDataAbsFields::Param3(self.param3),
                MyDataAbsKeys::Inner1(inner_key) => {
                    MyDataAbsFields::Inner1(self.inner1.get(inner_key))
                }
            }
        }

        fn set(&mut self, field: MyDataAbsFields) {
            match field {
                MyDataAbsFields::Param1(value) => self.param1 = value,
                MyDataAbsFields::Param2(value) => self.param2 = value,
                MyDataAbsFields::Param3(value) => self.param3 = value,
                MyDataAbsFields::Inner1(inner_field) => {
                    self.inner1.set(inner_field);
                }
            }
        }
    }
}
