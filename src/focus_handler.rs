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
pub(crate) mod test_data_focus_handler {
    use core::sync::atomic::{AtomicU8, Ordering};

    use crate::{
        FocusHandler, FolderFocus, ToFull,
        layer::{
            AbsFields, AbsKeys, Fields, Keys, MyInnerDataFocus, MyInnerDataFolderPath,
            MyInnerInnerDataFocus, MyInnerInnerDataFolderPath, my_inner_data, my_inner_inner_data,
        },
        test_types::TestLayerDatabaseDescription,
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
        fn get_focus_key(&self, key: Keys) -> AbsKeys {
            match key {
                Keys::Param1 => AbsKeys::Param1,
                Keys::Param2 => AbsKeys::Param2,
                Keys::Param3 => AbsKeys::Param3,
                Keys::Param4 | Keys::Param5 => {
                    let folder_path: MyInnerDataFolderPath = self.get_focus_path();
                    match key {
                        Keys::Param4 => folder_path.build_full(my_inner_data::Keys::Param4),
                        Keys::Param5 => folder_path.build_full(my_inner_data::Keys::Param5),
                        _ => unreachable!(),
                    }
                }
                Keys::Param6 => {
                    let folder_path: MyInnerInnerDataFolderPath = self.get_focus_path();
                    match key {
                        Keys::Param6 => folder_path.build_full(my_inner_inner_data::Keys::Param6),
                        _ => unreachable!(),
                    }
                }
            }
        }

        fn get_focus_field(&self, field: Fields) -> AbsFields {
            match field {
                Fields::Param1(value) => AbsFields::Param1(value),
                Fields::Param2(value) => AbsFields::Param2(value),
                Fields::Param3(value) => AbsFields::Param3(value),
                Fields::Param4(_) | Fields::Param5(_) => {
                    let folder_path: MyInnerDataFolderPath = self.get_focus_path();

                    match field {
                        Fields::Param4(value) => {
                            folder_path.build_full(my_inner_data::Fields::Param4(value))
                        }
                        Fields::Param5(value) => {
                            folder_path.build_full(my_inner_data::Fields::Param5(value))
                        }
                        _ => unreachable!(),
                    }
                }
                Fields::Param6(_) => {
                    let folder_path: MyInnerInnerDataFolderPath = self.get_focus_path();

                    match field {
                        Fields::Param6(value) => {
                            folder_path.build_full(my_inner_inner_data::Fields::Param6(value))
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
            MyInnerInnerDataFolderPath,
            (AbsKeys, AbsFields),
            (my_inner_inner_data::Keys, my_inner_inner_data::Fields),
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

        fn get_focus_path(&self) -> MyInnerInnerDataFolderPath {
            let inner_focus: MyInnerDataFocus = self.get_focus();

            match inner_focus {
                MyInnerDataFocus::Inner1 | MyInnerDataFocus::Inner2 => {
                    let inner_inner_focus: MyInnerInnerDataFocus = self.get_focus();

                    match inner_inner_focus {
                        MyInnerInnerDataFocus::Inner3 => match inner_focus {
                            MyInnerDataFocus::Inner1 => MyInnerInnerDataFolderPath::Inner1(
                                my_inner_data::MyInnerInnerDataFolderPath::Inner3,
                            ),
                            MyInnerDataFocus::Inner2 => MyInnerInnerDataFolderPath::Inner2(
                                my_inner_data::MyInnerInnerDataFolderPath::Inner3,
                            ),
                        },
                        MyInnerInnerDataFocus::Inner4 => match inner_focus {
                            MyInnerDataFocus::Inner1 => MyInnerInnerDataFolderPath::Inner1(
                                my_inner_data::MyInnerInnerDataFolderPath::Inner4,
                            ),
                            MyInnerDataFocus::Inner2 => MyInnerInnerDataFolderPath::Inner2(
                                my_inner_data::MyInnerInnerDataFolderPath::Inner4,
                            ),
                        },
                    }
                }
            }
        }
    }

    impl
        FolderFocus<
            MyInnerDataFocus,
            MyInnerDataFolderPath,
            (AbsKeys, AbsFields),
            (my_inner_data::Keys, my_inner_data::Fields),
        > for MyFocusHandler
    {
        fn get_focus(&self) -> MyInnerDataFocus {
            self.inner_focus.load(Ordering::SeqCst).try_into().unwrap()
        }

        fn set_focus(&self, focus: MyInnerDataFocus) {
            self.inner_focus.store(focus.into(), Ordering::SeqCst);
        }

        fn get_focus_path(&self) -> MyInnerDataFolderPath {
            match self.get_focus() {
                MyInnerDataFocus::Inner1 => MyInnerDataFolderPath::Inner1,
                MyInnerDataFocus::Inner2 => MyInnerDataFolderPath::Inner2,
            }
        }
    }
}
