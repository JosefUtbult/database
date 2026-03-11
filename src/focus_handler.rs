use crate::DatabaseDescription;

pub trait FocusHandler<Database: DatabaseDescription>
{
    fn get_focus_key(&self, key: Database::FlatKey) -> Database::AbsKey;
    fn get_focus_field(&self, field: Database::FlatField) -> Database::AbsField;
}

pub trait FolderFocusDescription<Database: DatabaseDescription>
{
}

pub trait FolderFocus<Database: DatabaseDescription>
{

}

#[cfg(test)]
pub(crate) mod test_data_field_accessor {
    use core::sync::atomic::{AtomicU8, Ordering};

    use crate::{
        FocusHandler,
        test_types::{
            TestLayerDatabaseDescription,
            layer::{
                MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields, MyDataFlatKeys,
                MyInnerDataFields, MyInnerDataKeys, MyInnerInnerDataFields, MyInnerInnerDataKeys,
            },
        },
    };

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum InnerFocus {
        One,
        Two,
    }

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum InnerInnerFocus {
        Three,
        Four,
    }

    impl From<InnerFocus> for u8 {
        fn from(value: InnerFocus) -> Self {
            match value {
                InnerFocus::One => 0,
                InnerFocus::Two => 1,
            }
        }
    }

    impl TryFrom<u8> for InnerFocus {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(InnerFocus::One),
                1 => Ok(InnerFocus::Two),
                _ => Err(()),
            }
        }
    }

    impl From<InnerInnerFocus> for u8 {
        fn from(value: InnerInnerFocus) -> Self {
            match value {
                InnerInnerFocus::Three => 0,
                InnerInnerFocus::Four => 1,
            }
        }
    }

    impl TryFrom<u8> for InnerInnerFocus {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(InnerInnerFocus::Three),
                1 => Ok(InnerInnerFocus::Four),
                _ => Err(()),
            }
        }
    }

    pub(crate) struct MyFocusHandler {
        inner_focus: AtomicU8,
        inner_inner_focus: AtomicU8,
    }

    impl MyFocusHandler {
        pub(crate) const fn new() -> Self {
            Self {
                inner_focus: AtomicU8::new(0),
                inner_inner_focus: AtomicU8::new(0),
            }
        }

        pub(crate) fn set_inner_focus(&self, inner_focus: InnerFocus) {
            self.inner_focus.store(inner_focus.into(), Ordering::SeqCst);
        }

        pub(crate) fn get_inner_focus(&self) -> InnerFocus {
            self.inner_focus.load(Ordering::SeqCst).try_into().unwrap()
        }

        #[allow(dead_code)]
        pub(crate) fn set_inner_inner_focus(&self, inner_inner_focus: InnerInnerFocus) {
            self.inner_inner_focus
                .store(inner_inner_focus.into(), Ordering::SeqCst);
        }

        pub(crate) fn get_inner_inner_focus(&self) -> InnerInnerFocus {
            self.inner_inner_focus
                .load(Ordering::SeqCst)
                .try_into()
                .unwrap()
        }
    }

    impl FocusHandler<TestLayerDatabaseDescription> for MyFocusHandler {
        fn get_focus_key(&self, key: MyDataFlatKeys) -> MyDataAbsKeys {
            match key {
                MyDataFlatKeys::Param1 => MyDataAbsKeys::Param1,
                MyDataFlatKeys::Param2 => MyDataAbsKeys::Param2,
                MyDataFlatKeys::Param3 => MyDataAbsKeys::Param3,
                MyDataFlatKeys::Param4 | MyDataFlatKeys::Param5 | MyDataFlatKeys::Param6 => {
                    match self.get_inner_focus() {
                        InnerFocus::One => match key {
                            MyDataFlatKeys::Param4 => {
                                MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4)
                            }
                            MyDataFlatKeys::Param5 => {
                                MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5)
                            }
                            MyDataFlatKeys::Param6 => match self.get_inner_inner_focus() {
                                InnerInnerFocus::Three => MyDataAbsKeys::Inner1(
                                    MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6),
                                ),
                                InnerInnerFocus::Four => MyDataAbsKeys::Inner1(
                                    MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6),
                                ),
                            },
                            _ => unreachable!(),
                        },
                        InnerFocus::Two => match key {
                            MyDataFlatKeys::Param4 => {
                                MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4)
                            }
                            MyDataFlatKeys::Param5 => {
                                MyDataAbsKeys::Inner2(MyInnerDataKeys::Param5)
                            }
                            MyDataFlatKeys::Param6 => match self.get_inner_inner_focus() {
                                InnerInnerFocus::Three => MyDataAbsKeys::Inner2(
                                    MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6),
                                ),
                                InnerInnerFocus::Four => MyDataAbsKeys::Inner2(
                                    MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6),
                                ),
                            },
                            _ => unreachable!(),
                        },
                    }
                }
            }
        }

        fn get_focus_field(&self, field: MyDataFlatFields) -> MyDataAbsFields {
            match field {
                MyDataFlatFields::Param1(value) => MyDataAbsFields::Param1(value),
                MyDataFlatFields::Param2(value) => MyDataAbsFields::Param2(value),
                MyDataFlatFields::Param3(value) => MyDataAbsFields::Param3(value),
                MyDataFlatFields::Param4(_)
                | MyDataFlatFields::Param5(_)
                | MyDataFlatFields::Param6(_) => match self.get_inner_focus() {
                    InnerFocus::One => match field {
                        MyDataFlatFields::Param4(value) => {
                            MyDataAbsFields::Inner1(MyInnerDataFields::Param4(value))
                        }
                        MyDataFlatFields::Param5(value) => {
                            MyDataAbsFields::Inner1(MyInnerDataFields::Param5(value))
                        }
                        MyDataFlatFields::Param6(value) => match self.get_inner_inner_focus() {
                            InnerInnerFocus::Three => MyDataAbsFields::Inner1(
                                MyInnerDataFields::Inner3(MyInnerInnerDataFields::Param6(value)),
                            ),
                            InnerInnerFocus::Four => MyDataAbsFields::Inner1(
                                MyInnerDataFields::Inner4(MyInnerInnerDataFields::Param6(value)),
                            ),
                        },
                        _ => unreachable!(),
                    },
                    InnerFocus::Two => match field {
                        MyDataFlatFields::Param4(value) => {
                            MyDataAbsFields::Inner2(MyInnerDataFields::Param4(value))
                        }
                        MyDataFlatFields::Param5(value) => {
                            MyDataAbsFields::Inner2(MyInnerDataFields::Param5(value))
                        }
                        MyDataFlatFields::Param6(value) => match self.get_inner_inner_focus() {
                            InnerInnerFocus::Three => MyDataAbsFields::Inner2(
                                MyInnerDataFields::Inner3(MyInnerInnerDataFields::Param6(value)),
                            ),
                            InnerInnerFocus::Four => MyDataAbsFields::Inner2(
                                MyInnerDataFields::Inner4(MyInnerInnerDataFields::Param6(value)),
                            ),
                        },
                        _ => unreachable!(),
                    },
                },
            }
        }
    }
}
