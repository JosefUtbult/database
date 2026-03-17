pub(crate) const TEST_LAYER_DATABASE_ABS_COUNT: usize = layer::MY_DATA_ABS_VARIANT_COUNT;
pub(crate) const TEST_LAYER_DATABASE_FLAT_COUNT: usize = layer::MY_DATA_FLAT_VARIANT_COUNT;

pub(crate) type TestLayerDatabaseDescription = (
    (layer::MyDataAbsKeys, layer::MyDataAbsFields),
    (layer::MyDataFlatKeys, layer::MyDataFlatFields),
    layer::MyLayerData,
);

pub(crate) mod layer {
    use crate::{
        AbsFieldConstraints, AbsKeyConstraints, AccessorError, AllVariants, DataFieldAccessor,
        DataFieldTryAccessor, DynamicKeySet, FlatFieldConstraints, FlatKeyConstraints,
        FocusConstraints, FolderHandler, PathConstraints, PathDescription, ToFromUsize, ToFull,
        ToKey, VariantCount, database_core::DatabaseError,
        test_types::TestLayerDatabaseDescription,
    };

    pub(crate) struct MyInnerInnerData {
        pub(crate) param6: u8,
    }

    impl MyInnerInnerData {
        pub(crate) const fn new() -> Self {
            Self { param6: 0 }
        }
    }

    pub(crate) struct MyInnerData {
        pub(crate) param4: u8,
        pub(crate) param5: bool,
        pub(crate) inner3: MyInnerInnerData,
        pub(crate) inner4: MyInnerInnerData,
    }

    impl MyInnerData {
        pub(crate) const fn new() -> Self {
            Self {
                param4: 0,
                param5: false,
                inner3: MyInnerInnerData::new(),
                inner4: MyInnerInnerData::new(),
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

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerInnerDataKeys {
        Param6,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerInnerDataFields {
        Param6(u8),
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataKeys {
        Param4,
        Param5,
        Inner3(MyInnerInnerDataKeys),
        Inner4(MyInnerInnerDataKeys),
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataFields {
        Param4(u8),
        Param5(bool),
        Inner3(MyInnerInnerDataFields),
        Inner4(MyInnerInnerDataFields),
    }

    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerData_MyInnerInnerData_FolderPath {
        Inner3,
        Inner4,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataAbsKeys {
        Param1,
        Param2,
        Param3,
        Inner1(MyInnerDataKeys),
        Inner2(MyInnerDataKeys),
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

    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerInnerDataFocus {
        Inner3,
        Inner4,
    }

    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyInnerDataFocus {
        Inner1,
        Inner2,
    }

    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyLayerData_MyInnerInnerData_FolderPath {
        Inner1(MyInnerData_MyInnerInnerData_FolderPath),
        Inner2(MyInnerData_MyInnerInnerData_FolderPath),
    }

    #[allow(dead_code)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyLayerData_MyInnerData_FolderPath {
        Inner1,
        Inner2,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataFlatKeys {
        Param1,
        Param2,
        Param3,
        Param4,
        Param5,
        Param6,
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
    pub(crate) enum MyDataFlatFields {
        Param1(u8),
        Param2(bool),
        Param3(u8),
        Param4(u8),
        Param5(bool),
        Param6(u8),
    }

    impl ToKey<MyInnerInnerDataKeys> for MyInnerInnerDataFields {
        fn to_key(&self) -> MyInnerInnerDataKeys {
            match self {
                MyInnerInnerDataFields::Param6(_) => MyInnerInnerDataKeys::Param6,
            }
        }
    }

    impl ToKey<MyInnerDataKeys> for MyInnerDataFields {
        fn to_key(&self) -> MyInnerDataKeys {
            match self {
                MyInnerDataFields::Param4(_) => MyInnerDataKeys::Param4,
                MyInnerDataFields::Param5(_) => MyInnerDataKeys::Param5,
                MyInnerDataFields::Inner3(field) => MyInnerDataKeys::Inner3(field.to_key()),
                MyInnerDataFields::Inner4(field) => MyInnerDataKeys::Inner4(field.to_key()),
            }
        }
    }

    impl ToKey<MyDataAbsKeys> for MyDataAbsFields {
        fn to_key(&self) -> MyDataAbsKeys {
            match self {
                MyDataAbsFields::Param1(_) => MyDataAbsKeys::Param1,
                MyDataAbsFields::Param2(_) => MyDataAbsKeys::Param2,
                MyDataAbsFields::Param3(_) => MyDataAbsKeys::Param3,
                MyDataAbsFields::Inner1(inner) => MyDataAbsKeys::Inner1(inner.to_key()),
                MyDataAbsFields::Inner2(inner) => MyDataAbsKeys::Inner2(inner.to_key()),
            }
        }
    }

    impl ToKey<MyDataFlatKeys> for MyDataFlatFields {
        fn to_key(&self) -> MyDataFlatKeys {
            match self {
                MyDataFlatFields::Param1(_) => MyDataFlatKeys::Param1,
                MyDataFlatFields::Param2(_) => MyDataFlatKeys::Param2,
                MyDataFlatFields::Param3(_) => MyDataFlatKeys::Param3,
                MyDataFlatFields::Param4(_) => MyDataFlatKeys::Param4,
                MyDataFlatFields::Param5(_) => MyDataFlatKeys::Param5,
                MyDataFlatFields::Param6(_) => MyDataFlatKeys::Param6,
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
                    MyInnerDataKeys::Inner3(inner) | MyInnerDataKeys::Inner4(inner) => {
                        match inner {
                            MyInnerInnerDataKeys::Param6 => MyDataFlatKeys::Param6,
                        }
                    }
                },
            }
        }
    }

    impl ToFromUsize for MyDataFlatKeys {
        fn to_usize(&self) -> usize {
            match self {
                Self::Param1 => 0,
                Self::Param2 => 1,
                Self::Param3 => 2,
                Self::Param4 => 3,
                Self::Param5 => 4,
                Self::Param6 => 5,
            }
        }

        fn try_from_usize(value: usize) -> Option<Self> {
            match value {
                0 => Some(Self::Param1),
                1 => Some(Self::Param2),
                2 => Some(Self::Param3),
                3 => Some(Self::Param4),
                4 => Some(Self::Param5),
                5 => Some(Self::Param6),
                _ => None,
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
                    MyInnerDataFields::Inner3(inner) | MyInnerDataFields::Inner4(inner) => {
                        match inner {
                            MyInnerInnerDataFields::Param6(value) => {
                                MyDataFlatFields::Param6(value)
                            }
                        }
                    }
                },
            }
        }
    }

    impl ToFull<MyInnerDataKeys, MyInnerInnerDataKeys> for MyInnerData_MyInnerInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerInnerDataKeys) -> MyInnerDataKeys {
            match self {
                Self::Inner3 => MyInnerDataKeys::Inner3(internal),
                Self::Inner4 => MyInnerDataKeys::Inner3(internal),
            }
        }
    }

    impl ToFull<MyInnerDataFields, MyInnerInnerDataFields> for MyInnerData_MyInnerInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerInnerDataFields) -> MyInnerDataFields {
            match self {
                Self::Inner3 => MyInnerDataFields::Inner3(internal),
                Self::Inner4 => MyInnerDataFields::Inner3(internal),
            }
        }
    }

    impl ToFull<MyDataAbsKeys, MyInnerDataKeys> for MyLayerData_MyInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerDataKeys) -> MyDataAbsKeys {
            match self {
                Self::Inner1 => MyDataAbsKeys::Inner1(internal),
                Self::Inner2 => MyDataAbsKeys::Inner2(internal),
            }
        }
    }

    impl ToFull<MyDataAbsFields, MyInnerDataFields> for MyLayerData_MyInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerDataFields) -> MyDataAbsFields {
            match self {
                Self::Inner1 => MyDataAbsFields::Inner1(internal),
                Self::Inner2 => MyDataAbsFields::Inner2(internal),
            }
        }
    }

    impl ToFull<MyDataAbsKeys, MyInnerInnerDataKeys> for MyLayerData_MyInnerInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerInnerDataKeys) -> MyDataAbsKeys {
            match self {
                Self::Inner1(inner) => MyDataAbsKeys::Inner1(inner.build_full(internal)),
                Self::Inner2(inner) => MyDataAbsKeys::Inner2(inner.build_full(internal)),
            }
        }
    }

    impl ToFull<MyDataAbsFields, MyInnerInnerDataFields> for MyLayerData_MyInnerInnerData_FolderPath {
        fn build_full(&self, internal: MyInnerInnerDataFields) -> MyDataAbsFields {
            match self {
                Self::Inner1(inner) => MyDataAbsFields::Inner1(inner.build_full(internal)),
                Self::Inner2(inner) => MyDataAbsFields::Inner2(inner.build_full(internal)),
            }
        }
    }

    impl From<MyInnerInnerDataFocus> for u8 {
        fn from(value: MyInnerInnerDataFocus) -> Self {
            match value {
                MyInnerInnerDataFocus::Inner3 => 0,
                MyInnerInnerDataFocus::Inner4 => 1,
            }
        }
    }

    impl TryFrom<u8> for MyInnerInnerDataFocus {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MyInnerInnerDataFocus::Inner3),
                1 => Ok(MyInnerInnerDataFocus::Inner4),
                _ => Err(()),
            }
        }
    }

    impl From<MyInnerDataFocus> for u8 {
        fn from(value: MyInnerDataFocus) -> Self {
            match value {
                MyInnerDataFocus::Inner1 => 0,
                MyInnerDataFocus::Inner2 => 1,
            }
        }
    }

    impl TryFrom<u8> for MyInnerDataFocus {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MyInnerDataFocus::Inner1),
                1 => Ok(MyInnerDataFocus::Inner2),
                _ => Err(()),
            }
        }
    }

    pub(crate) const MY_DATA_ABS_VARIANT_COUNT: usize = 11;
    pub(crate) const MY_DATA_ABS_KEYS_ALL_VARIANTS: [MyDataAbsKeys; MY_DATA_ABS_VARIANT_COUNT] = [
        MyDataAbsKeys::Param1,
        MyDataAbsKeys::Param2,
        MyDataAbsKeys::Param3,
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner1(MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Param5),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Inner3(MyInnerInnerDataKeys::Param6)),
        MyDataAbsKeys::Inner2(MyInnerDataKeys::Inner4(MyInnerInnerDataKeys::Param6)),
    ];

    impl VariantCount for MyDataAbsKeys {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    impl AllVariants for MyDataAbsKeys {
        const ALL_VARIANTS: &[Self] = &MY_DATA_ABS_KEYS_ALL_VARIANTS;
    }

    impl VariantCount for MyDataAbsFields {
        const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
    }

    pub(crate) const MY_DATA_FLAT_VARIANT_COUNT: usize = 6;

    impl VariantCount for MyDataFlatKeys {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl VariantCount for MyDataFlatFields {
        const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
    }

    impl AbsKeyConstraints for MyDataAbsKeys {}

    impl AbsFieldConstraints<MyDataAbsKeys> for MyDataAbsFields {}

    impl FlatKeyConstraints<MyDataAbsKeys> for MyDataFlatKeys {}

    impl FlatFieldConstraints<MyDataAbsFields, MyDataFlatKeys> for MyDataFlatFields {}

    impl FocusConstraints for MyInnerInnerDataFocus {}
    impl FocusConstraints for MyInnerDataFocus {}

    impl
        PathConstraints<
            (MyDataAbsKeys, MyDataAbsFields),
            (MyInnerInnerDataKeys, MyInnerInnerDataFields),
        > for MyLayerData_MyInnerInnerData_FolderPath
    {
    }
    impl PathConstraints<(MyDataAbsKeys, MyDataAbsFields), (MyInnerDataKeys, MyInnerDataFields)>
        for MyLayerData_MyInnerData_FolderPath
    {
    }

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
                MyInnerDataKeys::Inner4(key) => MyInnerDataFields::Inner4(self.inner4.get(key)),
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
                MyInnerDataKeys::Param4 => todo!(),
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
                MyInnerDataKeys::Inner4(key) => self.inner4.try_set(key, value),
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
            match key {
                MyDataAbsKeys::Param1 => MyDataAbsFields::Param1(self.param1.clone()),
                MyDataAbsKeys::Param2 => MyDataAbsFields::Param2(self.param2.clone()),
                MyDataAbsKeys::Param3 => MyDataAbsFields::Param3(self.param3.clone()),
                MyDataAbsKeys::Inner1(key) => MyDataAbsFields::Inner1(self.inner1.get(key)),
                MyDataAbsKeys::Inner2(key) => MyDataAbsFields::Inner2(self.inner2.get(key)),
            }
        }

        fn set(&mut self, field: MyDataAbsFields) {
            match field {
                MyDataAbsFields::Param1(value) => self.param1 = value,
                MyDataAbsFields::Param2(value) => self.param2 = value,
                MyDataAbsFields::Param3(value) => self.param3 = value,
                MyDataAbsFields::Inner1(field) => self.inner1.set(field),
                MyDataAbsFields::Inner2(field) => self.inner2.set(field),
            }
        }
    }

    impl DataFieldTryAccessor<MyDataAbsKeys, u8> for MyLayerData {
        fn try_get(&self, key: MyDataAbsKeys) -> Result<u8, AccessorError> {
            match key {
                MyDataAbsKeys::Param1 => Ok(self.param1.clone()),
                MyDataAbsKeys::Param3 => Ok(self.param3.clone()),
                MyDataAbsKeys::Inner1(key) => self.inner1.try_get(key),
                MyDataAbsKeys::Inner2(key) => self.inner2.try_get(key),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }

        fn try_set(&mut self, key: MyDataAbsKeys, value: u8) -> Result<(), AccessorError> {
            match key {
                MyDataAbsKeys::Param1 => {
                    self.param1 = value;
                    Ok(())
                }
                MyDataAbsKeys::Param3 => {
                    self.param3 = value;
                    Ok(())
                }
                MyDataAbsKeys::Inner1(key) => self.inner1.try_set(key, value),
                MyDataAbsKeys::Inner2(key) => self.inner2.try_set(key, value),
                _ => Err(AccessorError::TypeMissmatch("u8")),
            }
        }
    }

    impl DataFieldTryAccessor<MyDataAbsKeys, bool> for MyLayerData {
        fn try_get(&self, key: MyDataAbsKeys) -> Result<bool, AccessorError> {
            match key {
                MyDataAbsKeys::Param2 => Ok(self.param2.clone()),
                MyDataAbsKeys::Inner1(key) => self.inner1.try_get(key),
                MyDataAbsKeys::Inner2(key) => self.inner2.try_get(key),
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }

        fn try_set(&mut self, key: MyDataAbsKeys, value: bool) -> Result<(), AccessorError> {
            match key {
                MyDataAbsKeys::Param2 => {
                    self.param2 = value;
                    Ok(())
                }
                MyDataAbsKeys::Inner1(key) => self.inner1.try_set(key, value),
                MyDataAbsKeys::Inner2(key) => self.inner2.try_set(key, value),
                _ => Err(AccessorError::TypeMissmatch("bool")),
            }
        }
    }

    impl FolderHandler<(), TestLayerDatabaseDescription> for MyLayerData {
        type Content = Self;

        fn get_at<'a>(&'a self, _path: ()) -> &'a Self::Content {
            self
        }

        fn get_at_mut<'a>(&'a mut self, _path: ()) -> &'a mut Self::Content {
            self
        }

        fn compare(
            &self,
            differing_keys: &mut dyn DynamicKeySet<MyDataAbsKeys, MyDataFlatKeys>,
            _path: (),
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            if self.param1 != other.param1 {
                differing_keys.insert_flat_key(MyDataFlatKeys::Param1)?;
            }

            if self.param2 != other.param2 {
                differing_keys.insert_flat_key(MyDataFlatKeys::Param2)?;
            }

            if self.param3 != other.param3 {
                differing_keys.insert_flat_key(MyDataFlatKeys::Param3)?;
            }

            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<MyDataAbsKeys, MyDataFlatKeys>,
            _path: (),
            other: &Self::Content,
        ) -> Result<(), crate::database_core::DatabaseError> {
            if self.param1 != other.param1 {
                self.param1 = other.param1;
                differing_keys.insert_flat_key(MyDataFlatKeys::Param1)?;
            }

            if self.param2 != other.param2 {
                self.param2 = other.param2;
                differing_keys.insert_flat_key(MyDataFlatKeys::Param2)?;
            }

            if self.param3 != other.param3 {
                self.param3 = other.param3;
                differing_keys.insert_flat_key(MyDataFlatKeys::Param3)?;
            }

            Ok(())
        }
    }

    impl FolderHandler<MyLayerData_MyInnerData_FolderPath, TestLayerDatabaseDescription>
        for MyLayerData
    {
        type Content = MyInnerData;

        fn get_at<'a>(&'a self, path: MyLayerData_MyInnerData_FolderPath) -> &'a Self::Content {
            match path {
                MyLayerData_MyInnerData_FolderPath::Inner1 => &self.inner1,
                MyLayerData_MyInnerData_FolderPath::Inner2 => &self.inner2,
            }
        }

        fn get_at_mut<'a>(
            &'a mut self,
            path: MyLayerData_MyInnerData_FolderPath,
        ) -> &'a mut Self::Content {
            match path {
                MyLayerData_MyInnerData_FolderPath::Inner1 => &mut self.inner1,
                MyLayerData_MyInnerData_FolderPath::Inner2 => &mut self.inner2,
            }
        }

        fn compare(
            &self,
            _differing_keys: &mut dyn DynamicKeySet<MyDataAbsKeys, MyDataFlatKeys>,
            _path: MyLayerData_MyInnerData_FolderPath,
            _other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            todo!()
        }

        fn clone(
            &mut self,
            _differing_keys: &mut dyn DynamicKeySet<MyDataAbsKeys, MyDataFlatKeys>,
            _path: MyLayerData_MyInnerData_FolderPath,
            _other: &Self::Content,
        ) -> Result<(), crate::database_core::DatabaseError> {
            todo!()
        }
    }
}
