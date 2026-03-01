use crate::{DatabaseDescription, UsizeConstraints};

pub trait FocusHandler<Database: DatabaseDescription>
where
    usize: UsizeConstraints<<Database as DatabaseDescription>::FlatKey>,
{
    fn get_focus_key(&self, key: Database::FlatKey) -> Database::AbsKey;
    fn get_focus_field(&self, field: Database::FlatField) -> Database::AbsField;
}

#[cfg(test)]
pub(crate) mod test_data_field_accessor {
    use core::cell::RefCell;

    use crate::{
        test::MyDatabaseDescription, test_types::test_types::{
            MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields, MyDataFlatKeys, MyInnerDataFields,
            MyInnerDataKeys,
        }, FocusHandler
    };

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum InnerFocus {
        One,
        Two,
    }

    pub(crate) struct MyFocusHandler {
        inner_focus: std::sync::Mutex<RefCell<InnerFocus>>,
    }

    impl MyFocusHandler {
        pub(crate) const fn new() -> Self {
            Self {
                inner_focus: std::sync::Mutex::new(RefCell::new(InnerFocus::One)),
            }
        }

        pub(crate) fn set_inner_focus(&self, focus: InnerFocus) {
            let lock = self.inner_focus.lock().unwrap();
            let mut inner_focus = lock.borrow_mut();
            *inner_focus = focus;
        }
    }

    impl FocusHandler<MyDatabaseDescription>
        for MyFocusHandler
    {
        fn get_focus_key(&self, key: MyDataFlatKeys) -> MyDataAbsKeys {
            let inner_focus = *self.inner_focus.lock().unwrap().borrow();
            let abs_key = match key {
                MyDataFlatKeys::Param1 => MyDataAbsKeys::Param1,
                MyDataFlatKeys::Param2 => MyDataAbsKeys::Param2,
                MyDataFlatKeys::Param3 => MyDataAbsKeys::Param3,
                MyDataFlatKeys::Param4 => match inner_focus {
                    InnerFocus::One => MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4),
                    InnerFocus::Two => MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4),
                },
                MyDataFlatKeys::Param5 => match inner_focus {
                    InnerFocus::One => MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5),
                    InnerFocus::Two => MyDataAbsKeys::Inner2(MyInnerDataKeys::Param5),
                },
            };

            std::println!(
                "Parsed focused key {:?} -> {:?}, inner focus {:?}",
                key,
                abs_key,
                inner_focus
            );

            abs_key
        }

        fn get_focus_field(&self, field: MyDataFlatFields) -> MyDataAbsFields {
            let inner_focus = *self.inner_focus.lock().unwrap().borrow();

            let abs_field = match field {
                MyDataFlatFields::Param1(value) => MyDataAbsFields::Param1(value),
                MyDataFlatFields::Param2(value) => MyDataAbsFields::Param2(value),
                MyDataFlatFields::Param3(value) => MyDataAbsFields::Param3(value),
                MyDataFlatFields::Param4(value) => match inner_focus {
                    InnerFocus::One => MyDataAbsFields::Inner1(MyInnerDataFields::Param4(value)),
                    InnerFocus::Two => MyDataAbsFields::Inner2(MyInnerDataFields::Param4(value)),
                },
                MyDataFlatFields::Param5(value) => match inner_focus {
                    InnerFocus::One => MyDataAbsFields::Inner1(MyInnerDataFields::Param5(value)),
                    InnerFocus::Two => MyDataAbsFields::Inner2(MyInnerDataFields::Param5(value)),
                },
            };

            std::println!(
                "Parsed focused field {:?} -> {:?}, inner focus {:?}",
                field,
                abs_field,
                inner_focus
            );

            abs_field
        }
    }
}
