pub trait DataFieldAccessor<AbsKey, AbsField> {
    fn get(&self, key: AbsKey) -> AbsField;
    fn set(&mut self, field: AbsField);
}

#[cfg(test)]
pub(crate) mod data_field_accessors {
    use crate::{
        DataFieldAccessor,
        test_types::test_types::{
            MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields, MyDataFlatKeys, MyFlatData,
            MyInnerData, MyInnerDataFields, MyInnerDataKeys, MyLayerData,
        },
    };

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

    impl DataFieldAccessor<MyDataAbsKeys, MyDataAbsFields> for MyLayerData {
        fn get(&self, key: MyDataAbsKeys) -> MyDataAbsFields {
            std::println!("Got get with abs key {:?}", key);
            match key {
                MyDataAbsKeys::Param1 => MyDataAbsFields::Param1(self.param1),
                MyDataAbsKeys::Param2 => MyDataAbsFields::Param2(self.param2),
                MyDataAbsKeys::Param3 => MyDataAbsFields::Param3(self.param3),
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
                MyDataFlatKeys::Param1 => MyDataFlatFields::Param1(self.param1),
                MyDataFlatKeys::Param2 => MyDataFlatFields::Param2(self.param2),
                MyDataFlatKeys::Param3 => MyDataFlatFields::Param3(self.param3),
                MyDataFlatKeys::Param4 => MyDataFlatFields::Param4(self.param4),
                MyDataFlatKeys::Param5 => MyDataFlatFields::Param5(self.param5),
            }
        }

        fn set(&mut self, field: MyDataFlatFields) {
            match field {
                MyDataFlatFields::Param1(value) => self.param1 = value,
                MyDataFlatFields::Param2(value) => self.param2 = value,
                MyDataFlatFields::Param3(value) => self.param3 = value,
                MyDataFlatFields::Param4(value) => self.param4 = value,
                MyDataFlatFields::Param5(value) => self.param5 = value,
            }
        }
    }
}
