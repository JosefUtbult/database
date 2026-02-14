#[cfg(test)]
pub(crate) mod test_types {
    use crate::DataFieldAccessor;

    pub(crate) struct MyData {
        param1: u8,
        param2: bool,
        param3: u8,
    }

    impl MyData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) enum MyDataKeys {
        Param1,
        Param2,
        Param3
    }

    impl From<MyDataKeys> for usize {
        fn from(value: MyDataKeys) -> Self {
            match value {
                MyDataKeys::Param1 => 0,
                MyDataKeys::Param2 => 1,
                MyDataKeys::Param3 => 2,
            }
        }
    }

    pub(crate) const MY_DATA_PARAMETER_COUNT: usize = 2;

    #[allow(dead_code)]
    pub(crate) const ALL_MY_DATA_KEYS: [MyDataKeys; MY_DATA_PARAMETER_COUNT] = [MyDataKeys::Param1, MyDataKeys::Param2];

    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum MyDataFields {
        Param1(u8),
        Param2(bool),
        Param3(u8)
    }

    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum MyDataError {
        Invalid,
    }

    impl TryFrom<MyDataFields> for u8 {
        type Error = MyDataError;

        fn try_from(value: MyDataFields) -> Result<Self, Self::Error> {
            match value {
                MyDataFields::Param1(value) => Ok(value),
                MyDataFields::Param3(value) => Ok(value),
                _ => Err(MyDataError::Invalid),
            }
        }
    }

    impl TryFrom<MyDataFields> for bool {
        type Error = MyDataError;

        fn try_from(value: MyDataFields) -> Result<Self, Self::Error> {
            match value {
                MyDataFields::Param2(value) => Ok(value),
                _ => Err(MyDataError::Invalid),
            }
        }
    }

    impl DataFieldAccessor<MyDataKeys, MyDataFields> for MyData {
        fn get(&self, key: MyDataKeys) -> MyDataFields {
            match key {
                MyDataKeys::Param1 => MyDataFields::Param1(self.param1),
                MyDataKeys::Param2 => MyDataFields::Param2(self.param2),
                MyDataKeys::Param3 => MyDataFields::Param3(self.param3),
            }
        }

        fn set(&mut self, field: MyDataFields) {
            match field {
                MyDataFields::Param1(value) => self.param1 = value,
                MyDataFields::Param2(value) => self.param2 = value,
                MyDataFields::Param3(value) => self.param3 = value,
            }
        }
    }
}
