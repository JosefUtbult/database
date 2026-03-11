use crate::{DatabaseDescription, Pair};

pub trait FocusHandler<Database: DatabaseDescription> {
    fn get_focus_key(&self, key: Database::FlatKey) -> Database::AbsKey;
    fn get_focus_field(&self, field: Database::FlatField) -> Database::AbsField;
}

pub trait FocusConstraints: Eq + Clone + Copy {}

pub trait ToFull<AbsEnum, InternalAbsEnum> {
    fn build_full(&self, internal: InternalAbsEnum) -> AbsEnum;
}

pub trait PathConstraints<AbsPair: Pair, InternalAbsPair: Pair>:
    Eq
    + Clone
    + Copy
    + ToFull<AbsPair::Key, InternalAbsPair::Key>
    + ToFull<AbsPair::Field, InternalAbsPair::Field>
{
}

pub trait FolderFocus<
    Focus: FocusConstraints,
    Path: PathConstraints<AbsPair, InternalAbsPair>,
    AbsPair: Pair,
    InternalAbsPair: Pair,
>
{
    fn get_focus(&self) -> Focus;
    fn set_focus(&self, focus: Focus);
    fn get_focus_path(&self) -> Path;
}

#[cfg(test)]
pub(crate) mod test_data_field_accessor {
    use core::sync::atomic::{AtomicU8, Ordering};

    use crate::{
        FocusHandler, FolderFocus, ToFull,
        test_types::{
            TestLayerDatabaseDescription,
            layer::{
                MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields, MyDataFlatKeys,
                MyInnerData_MyInnerInnerData_FolderPath, MyInnerDataFields, MyInnerDataFocus,
                MyInnerDataKeys, MyInnerInnerDataFields, MyInnerInnerDataFocus,
                MyInnerInnerDataKeys, MyLayerData_MyInnerData_FolderPath,
                MyLayerData_MyInnerInnerData_FolderPath,
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
    }

    impl FocusHandler<TestLayerDatabaseDescription> for MyFocusHandler {
        fn get_focus_key(&self, key: MyDataFlatKeys) -> MyDataAbsKeys {
            match key {
                MyDataFlatKeys::Param1 => MyDataAbsKeys::Param1,
                MyDataFlatKeys::Param2 => MyDataAbsKeys::Param2,
                MyDataFlatKeys::Param3 => MyDataAbsKeys::Param3,
                MyDataFlatKeys::Param4 | MyDataFlatKeys::Param5 => {
                    let folder_path: MyLayerData_MyInnerData_FolderPath = self.get_focus_path();
                    match key {
                        MyDataFlatKeys::Param4 => folder_path.build_full(MyInnerDataKeys::Param4),
                        MyDataFlatKeys::Param5 => folder_path.build_full(MyInnerDataKeys::Param5),
                        _ => unreachable!(),
                    }
                }
                MyDataFlatKeys::Param6 => {
                    let folder_path: MyLayerData_MyInnerInnerData_FolderPath =
                        self.get_focus_path();
                    match key {
                        MyDataFlatKeys::Param6 => {
                            folder_path.build_full(MyInnerInnerDataKeys::Param6)
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        fn get_focus_field(&self, field: MyDataFlatFields) -> MyDataAbsFields {
            match field {
                MyDataFlatFields::Param1(value) => MyDataAbsFields::Param1(value),
                MyDataFlatFields::Param2(value) => MyDataAbsFields::Param2(value),
                MyDataFlatFields::Param3(value) => MyDataAbsFields::Param3(value),
                MyDataFlatFields::Param4(_) | MyDataFlatFields::Param5(_) => {
                    let folder_path: MyLayerData_MyInnerData_FolderPath = self.get_focus_path();

                    match field {
                        MyDataFlatFields::Param4(value) => {
                            folder_path.build_full(MyInnerDataFields::Param4(value))
                        }
                        MyDataFlatFields::Param5(value) => {
                            folder_path.build_full(MyInnerDataFields::Param5(value))
                        }
                        _ => unreachable!(),
                    }
                }
                MyDataFlatFields::Param6(_) => {
                    let folder_path: MyLayerData_MyInnerInnerData_FolderPath =
                        self.get_focus_path();

                    match field {
                        MyDataFlatFields::Param6(value) => {
                            folder_path.build_full(MyInnerInnerDataFields::Param6(value))
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    impl
        FolderFocus<
            MyInnerInnerDataFocus,
            MyLayerData_MyInnerInnerData_FolderPath,
            (MyDataAbsKeys, MyDataAbsFields),
            (MyInnerInnerDataKeys, MyInnerInnerDataFields),
        > for MyFocusHandler
    {
        fn get_focus(&self) -> MyInnerInnerDataFocus {
            self.inner_inner_focus
                .load(Ordering::SeqCst)
                .try_into()
                .unwrap()
        }

        fn set_focus(&self, focus: MyInnerInnerDataFocus) {
            self.inner_inner_focus.store(focus.into(), Ordering::SeqCst);
        }

        fn get_focus_path(&self) -> MyLayerData_MyInnerInnerData_FolderPath {
            let inner_focus: MyInnerDataFocus = self.get_focus();

            match inner_focus {
                MyInnerDataFocus::Inner1 | MyInnerDataFocus::Inner2 => {
                    let inner_inner_focus: MyInnerInnerDataFocus = self.get_focus();

                    match inner_inner_focus {
                        MyInnerInnerDataFocus::Inner3 => {
                            match inner_focus {
                                MyInnerDataFocus::Inner1 => {
                                    MyLayerData_MyInnerInnerData_FolderPath::Inner1(
                                        MyInnerData_MyInnerInnerData_FolderPath::Inner3,
                                    )
                                },
                                MyInnerDataFocus::Inner2 => {
                                    MyLayerData_MyInnerInnerData_FolderPath::Inner2(
                                        MyInnerData_MyInnerInnerData_FolderPath::Inner3,
                                    )
                                },
                            }
                        }
                        MyInnerInnerDataFocus::Inner4 => {
                            match inner_focus {
                                MyInnerDataFocus::Inner1 => {
                                    MyLayerData_MyInnerInnerData_FolderPath::Inner1(
                                        MyInnerData_MyInnerInnerData_FolderPath::Inner4,
                                    )
                                },
                                MyInnerDataFocus::Inner2 => {
                                    MyLayerData_MyInnerInnerData_FolderPath::Inner2(
                                        MyInnerData_MyInnerInnerData_FolderPath::Inner4,
                                    )
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    impl
        FolderFocus<
            MyInnerDataFocus,
            MyLayerData_MyInnerData_FolderPath,
            (MyDataAbsKeys, MyDataAbsFields),
            (MyInnerDataKeys, MyInnerDataFields),
        > for MyFocusHandler
    {
        fn get_focus(&self) -> MyInnerDataFocus {
            self.inner_focus.load(Ordering::SeqCst).try_into().unwrap()
        }

        fn set_focus(&self, focus: MyInnerDataFocus) {
            self.inner_focus.store(focus.into(), Ordering::SeqCst);
        }

        fn get_focus_path(&self) -> MyLayerData_MyInnerData_FolderPath {
            match self.get_focus() {
                MyInnerDataFocus::Inner1 => MyLayerData_MyInnerData_FolderPath::Inner1,
                MyInnerDataFocus::Inner2 => MyLayerData_MyInnerData_FolderPath::Inner2,
            }
        }
    }
}
