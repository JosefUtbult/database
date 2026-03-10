pub enum AccessorError {
    TypeMissmatch(&'static str),
}

pub trait DataFieldAccessor<AbsKey, AbsField> {
    fn get(&self, key: AbsKey) -> AbsField;
    fn set(&mut self, field: AbsField);
}

pub trait DataFieldTryAccessor<AbsKey, T> {
    fn try_get(&self, key: AbsKey) -> Result<T, AccessorError>;
    fn try_set(&mut self, key: AbsKey, value: T) -> Result<(), AccessorError>;
}

pub trait FolderAccessor<T, AbsFolder> {
    fn try_get(&self, folder: AbsFolder) -> Result<&T, AccessorError>;
    fn try_get_mut(&mut self, folder: AbsFolder) -> Result<&mut T, AccessorError>;
}

#[cfg(test)]
pub(crate) mod data_field_accessors {
    use crate::{
        AccessorError, DataFieldAccessor, DataFieldTryAccessor,
        test_types::test_types::{
            MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields, MyDataFlatKeys, MyFlatData,
            MyInnerData, MyInnerDataFields, MyInnerDataKeys, MyInnerInnerData,
            MyInnerInnerDataFields, MyInnerInnerDataKeys, MyLayerData,
        },
    };

    impl DataFieldAccessor<MyInnerInnerDataKeys, MyInnerInnerDataFields> for MyInnerInnerData {
        fn get(&self, key: MyInnerInnerDataKeys) -> MyInnerInnerDataFields {
            match key {
                MyInnerInnerDataKeys::Param6 => MyInnerInnerDataFields::Param6(self.param6.clone()),
            }
        }

        fn set(&mut self, field: MyInnerInnerDataFields) {
            match field {
                MyInnerInnerDataFields::Param6(value) => self.param6 = value,
            }
        }
    }

    impl DataFieldTryAccessor<MyInnerInnerDataKeys, u8> for MyInnerInnerData {
        fn try_get(&self, key: MyInnerInnerDataKeys) -> Result<u8, AccessorError> {
            match key {
                MyInnerInnerDataKeys::Param6 => Ok(self.param6.clone()),
            }
        }

        fn try_set(&mut self, key: MyInnerInnerDataKeys, value: u8) -> Result<(), AccessorError> {
            match key {
                MyInnerInnerDataKeys::Param6 => {
                    self.param6 = value;
                    Ok(())
                }
            }
        }
    }

    impl DataFieldAccessor<MyInnerDataKeys, MyInnerDataFields> for MyInnerData {
        fn get(&self, key: MyInnerDataKeys) -> MyInnerDataFields {
            match key {
                MyInnerDataKeys::Param4 => MyInnerDataFields::Param4(self.param4.clone()),
                MyInnerDataKeys::Param5 => MyInnerDataFields::Param5(self.param5.clone()),
                MyInnerDataKeys::Inner3(key) => MyInnerDataFields::Inner3(self.inner3.get(key)),
                MyInnerDataKeys::Inner4(key) => MyInnerDataFields::Inner3(self.inner4.get(key)),
            }
        }

        fn set(&mut self, field: MyInnerDataFields) {
            match field {
                MyInnerDataFields::Param4(value) => self.param4 = value,
                MyInnerDataFields::Param5(value) => self.param5 = value,
                MyInnerDataFields::Inner3(field) => self.inner3.set(field),
                MyInnerDataFields::Inner4(field) => self.inner4.set(field),
            }
        }
    }

    impl DataFieldTryAccessor<MyInnerDataKeys, u8> for MyInnerData {
        fn try_get(&self, key: MyInnerDataKeys) -> Result<u8, AccessorError> {
            match key {
                MyInnerDataKeys::Param4 => Ok(self.param4.clone()),
                MyInnerDataKeys::Inner3(key) => self.inner3.try_get(key),
                MyInnerDataKeys::Inner4(key) => self.inner4.try_get(key),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }

        fn try_set(&mut self, key: MyInnerDataKeys, value: u8) -> Result<(), AccessorError> {
            match key {
                MyInnerDataKeys::Param4 => {
                    self.param4 = value;
                    Ok(())
                }
                MyInnerDataKeys::Inner3(key) => self.inner3.try_set(key, value),
                MyInnerDataKeys::Inner4(key) => self.inner3.try_set(key, value),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }
    }

    impl DataFieldTryAccessor<MyInnerDataKeys, bool> for MyInnerData {
        fn try_get(&self, key: MyInnerDataKeys) -> Result<bool, AccessorError> {
            match key {
                MyInnerDataKeys::Param5 => Ok(self.param5.clone()),
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }

        fn try_set(&mut self, key: MyInnerDataKeys, value: bool) -> Result<(), AccessorError> {
            match key {
                MyInnerDataKeys::Param5 => {
                    self.param5 = value;
                    Ok(())
                }
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }
    }

    impl DataFieldAccessor<MyDataAbsKeys, MyDataAbsFields> for MyLayerData {
        fn get(&self, key: MyDataAbsKeys) -> MyDataAbsFields {
            std::println!("Got get with abs key {:?}", key);
            match key {
                MyDataAbsKeys::Param1 => MyDataAbsFields::Param1(self.param1.clone()),
                MyDataAbsKeys::Param2 => MyDataAbsFields::Param2(self.param2.clone()),
                MyDataAbsKeys::Param3 => MyDataAbsFields::Param3(self.param3.clone()),
                MyDataAbsKeys::Inner1(inner_key) => {
                    MyDataAbsFields::Inner1(self.inner1.get(inner_key))
                }
                MyDataAbsKeys::Inner2(inner_key) => {
                    MyDataAbsFields::Inner2(self.inner2.get(inner_key))
                }
            }
        }

        fn set(&mut self, field: MyDataAbsFields) {
            std::println!("Got set with abs field {:?}", field);
            match field {
                MyDataAbsFields::Param1(value) => self.param1 = value,
                MyDataAbsFields::Param2(value) => self.param2 = value,
                MyDataAbsFields::Param3(value) => self.param3 = value,
                MyDataAbsFields::Inner1(inner_field) => {
                    self.inner1.set(inner_field);
                }
                MyDataAbsFields::Inner2(inner_field) => {
                    self.inner2.set(inner_field);
                }
            }
        }
    }

    impl DataFieldAccessor<MyDataFlatKeys, MyDataFlatFields> for MyFlatData {
        fn get(&self, key: MyDataFlatKeys) -> MyDataFlatFields {
            match key {
                MyDataFlatKeys::Param1 => MyDataFlatFields::Param1(self.param1.clone()),
                MyDataFlatKeys::Param2 => MyDataFlatFields::Param2(self.param2.clone()),
                MyDataFlatKeys::Param3 => MyDataFlatFields::Param3(self.param3.clone()),
                MyDataFlatKeys::Param4 => MyDataFlatFields::Param4(self.param4.clone()),
                MyDataFlatKeys::Param5 => MyDataFlatFields::Param5(self.param5.clone()),
                MyDataFlatKeys::Param6 => MyDataFlatFields::Param6(self.param6.clone()),
            }
        }

        fn set(&mut self, field: MyDataFlatFields) {
            match field {
                MyDataFlatFields::Param1(value) => self.param1 = value,
                MyDataFlatFields::Param2(value) => self.param2 = value,
                MyDataFlatFields::Param3(value) => self.param3 = value,
                MyDataFlatFields::Param4(value) => self.param4 = value,
                MyDataFlatFields::Param5(value) => self.param5 = value,
                MyDataFlatFields::Param6(value) => self.param6 = value,
            }
        }
    }

    impl DataFieldTryAccessor<MyDataFlatKeys, u8> for MyFlatData {
        fn try_get(&self, key: MyDataFlatKeys) -> Result<u8, AccessorError> {
            match key {
                MyDataFlatKeys::Param1 => Ok(self.param1.clone()),
                MyDataFlatKeys::Param3 => Ok(self.param3.clone()),
                MyDataFlatKeys::Param4 => Ok(self.param4.clone()),
                MyDataFlatKeys::Param6 => Ok(self.param6.clone()),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }

        fn try_set(&mut self, key: MyDataFlatKeys, value: u8) -> Result<(), AccessorError> {
            match key {
                MyDataFlatKeys::Param1 => {
                    self.param1 = value;
                    Ok(())
                }
                MyDataFlatKeys::Param3 => {
                    self.param3 = value;
                    Ok(())
                }
                MyDataFlatKeys::Param4 => {
                    self.param4 = value;
                    Ok(())
                }

                MyDataFlatKeys::Param6 => {
                    self.param6 = value;
                    Ok(())
                }
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }
    }

    impl DataFieldTryAccessor<MyDataFlatKeys, bool> for MyFlatData {
        fn try_get(&self, key: MyDataFlatKeys) -> Result<bool, AccessorError> {
            match key {
                MyDataFlatKeys::Param2 => Ok(self.param2.clone()),
                MyDataFlatKeys::Param5 => Ok(self.param5.clone()),
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }

        fn try_set(&mut self, key: MyDataFlatKeys, value: bool) -> Result<(), AccessorError> {
            match key {
                MyDataFlatKeys::Param2 => {
                    self.param2 = value;
                    Ok(())
                }
                MyDataFlatKeys::Param5 => {
                    self.param5 = value;
                    Ok(())
                }
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }
    }
}
