pub(crate) const TEST_FLAT_DATABASE_COUNT: usize = flat::FLAT_COUNT;

pub(crate) fn create_test_flat_data() -> flat::MyFlatData {
    flat::MyFlatData::new()
}

pub(crate) type TestFlatDatabaseDescription =
    ((flat::MyFlatKeys, flat::MyFlatFields), flat::MyFlatData);

pub(crate) mod flat {
    use crate::{
        AbsFieldConstraints, AbsKeyConstraints, AccessorError, AllVariants, DataFieldAccessor,
        DataFieldTryAccessor, FlatFieldConstraints, FlatKeyConstraints, Folder, ToFromUsize, ToKey,
        VariantCount, test_types::TestFlatDatabaseDescription,
    };

    pub(crate) struct MyFlatData {
        pub(crate) param1: u8,
        pub(crate) param2: bool,
        pub(crate) param3: u8,
        pub(crate) param4: u8,
    }

    impl MyFlatData {
        pub(crate) const fn new() -> Self {
            Self {
                param1: 0,
                param2: false,
                param3: 0,
                param4: 0,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyFlatKeys {
        Param1,
        Param2,
        Param3,
        Param4,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyFlatFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Param4(u8),
    }

    impl ToKey<MyFlatKeys> for MyFlatFields {
        fn to_key(&self) -> MyFlatKeys {
            match self {
                MyFlatFields::Param1(_) => MyFlatKeys::Param1,
                MyFlatFields::Param2(_) => MyFlatKeys::Param2,
                MyFlatFields::Param3(_) => MyFlatKeys::Param3,
                MyFlatFields::Param4(_) => MyFlatKeys::Param4,
            }
        }
    }

    impl From<MyFlatKeys> for usize {
        fn from(value: MyFlatKeys) -> Self {
            match value {
                MyFlatKeys::Param1 => 0,
                MyFlatKeys::Param2 => 1,
                MyFlatKeys::Param3 => 2,
                MyFlatKeys::Param4 => 3,
            }
        }
    }

    impl ToFromUsize for MyFlatKeys {
        fn to_usize(&self) -> usize {
            match self {
                Self::Param1 => 0,
                Self::Param2 => 1,
                Self::Param3 => 2,
                Self::Param4 => 3,
            }
        }

        fn try_from_usize(value: usize) -> Option<Self> {
            match value {
                0 => Some(Self::Param1),
                1 => Some(Self::Param2),
                2 => Some(Self::Param3),
                3 => Some(Self::Param4),
                _ => None,
            }
        }
    }

    pub(crate) const FLAT_COUNT: usize = 4;

    impl VariantCount for MyFlatKeys {
        const COUNT: usize = FLAT_COUNT;
    }

    impl AllVariants for MyFlatKeys {
        const ALL_VARIANTS: &[Self] = &[Self::Param1, Self::Param2, Self::Param3, Self::Param4];
    }

    impl VariantCount for MyFlatFields {
        const COUNT: usize = FLAT_COUNT;
    }

    impl AbsKeyConstraints for MyFlatKeys {}
    impl FlatKeyConstraints<MyFlatKeys> for MyFlatKeys {}

    impl AbsFieldConstraints<MyFlatKeys> for MyFlatFields {}
    impl FlatFieldConstraints<MyFlatFields, MyFlatKeys> for MyFlatFields {}

    impl DataFieldAccessor<MyFlatKeys, MyFlatFields> for MyFlatData {
        fn get(&self, key: MyFlatKeys) -> MyFlatFields {
            match key {
                MyFlatKeys::Param1 => MyFlatFields::Param1(self.param1),
                MyFlatKeys::Param2 => MyFlatFields::Param2(self.param2),
                MyFlatKeys::Param3 => MyFlatFields::Param3(self.param3),
                MyFlatKeys::Param4 => MyFlatFields::Param4(self.param4),
            }
        }

        fn set(&mut self, field: MyFlatFields) {
            match field {
                MyFlatFields::Param1(value) => self.param1 = value,
                MyFlatFields::Param2(value) => self.param2 = value,
                MyFlatFields::Param3(value) => self.param3 = value,
                MyFlatFields::Param4(value) => self.param4 = value,
            }
        }
    }

    impl DataFieldTryAccessor<MyFlatKeys, u8> for MyFlatData {
        fn try_get(&self, key: MyFlatKeys) -> Result<u8, AccessorError> {
            match key {
                MyFlatKeys::Param1 => Ok(self.param1.clone()),
                MyFlatKeys::Param3 => Ok(self.param3.clone()),
                MyFlatKeys::Param4 => Ok(self.param4.clone()),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }

        fn try_set(&mut self, key: MyFlatKeys, value: u8) -> Result<(), AccessorError> {
            match key {
                MyFlatKeys::Param1 => {
                    self.param1 = value;
                    Ok(())
                }
                MyFlatKeys::Param3 => {
                    self.param3 = value;
                    Ok(())
                }
                MyFlatKeys::Param4 => {
                    self.param4 = value;
                    Ok(())
                }
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }
    }

    impl DataFieldTryAccessor<MyFlatKeys, bool> for MyFlatData {
        fn try_get(&self, key: MyFlatKeys) -> Result<bool, AccessorError> {
            match key {
                MyFlatKeys::Param2 => Ok(self.param2.clone()),
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }

        fn try_set(&mut self, key: MyFlatKeys, value: bool) -> Result<(), AccessorError> {
            match key {
                MyFlatKeys::Param2 => {
                    self.param2 = value;
                    Ok(())
                }
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }
    }

    impl Folder<TestFlatDatabaseDescription> for MyFlatData {
        fn compare(
            &self,
            differing_keys: &mut dyn crate::DynamicKeySet<MyFlatKeys, MyFlatKeys>,
            other: &Self,
        ) -> Result<(), crate::DatabaseError> {
            if self.param1 != other.param1 {
                differing_keys.insert_flat_key(MyFlatKeys::Param1)?;
            }

            if self.param2 != other.param2 {
                differing_keys.insert_flat_key(MyFlatKeys::Param2)?;
            }

            if self.param3 != other.param3 {
                differing_keys.insert_flat_key(MyFlatKeys::Param3)?;
            }

            if self.param4 != other.param4 {
                differing_keys.insert_flat_key(MyFlatKeys::Param4)?;
            }

            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn crate::DynamicKeySet<MyFlatKeys, MyFlatKeys>,
            other: &Self,
        ) -> Result<(), crate::DatabaseError> {
            if self.param1 != other.param1 {
                self.param1 = other.param1;
                differing_keys.insert_flat_key(MyFlatKeys::Param1)?;
            }

            if self.param2 != other.param2 {
                self.param2 = other.param2;
                differing_keys.insert_flat_key(MyFlatKeys::Param2)?;
            }

            if self.param3 != other.param3 {
                self.param3 = other.param3;
                differing_keys.insert_flat_key(MyFlatKeys::Param3)?;
            }

            if self.param4 != other.param4 {
                self.param4 = other.param4;
                differing_keys.insert_flat_key(MyFlatKeys::Param4)?;
            }

            Ok(())
        }
    }
}
