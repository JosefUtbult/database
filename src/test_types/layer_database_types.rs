pub(crate) const TEST_LAYER_DATABASE_ABS_COUNT: usize =
    layer::my_layer_data::MY_DATA_ABS_VARIANT_COUNT;

pub(crate) const TEST_LAYER_DATABASE_FLAT_COUNT: usize =
    layer::my_layer_data::MY_DATA_FLAT_VARIANT_COUNT;

pub(crate) type TestLayerDatabaseDescription = (
    (
        layer::my_layer_data::AbsKeys,
        layer::my_layer_data::AbsFields,
    ),
    (layer::my_layer_data::Keys, layer::my_layer_data::Fields),
    layer::MyLayerData,
);

pub(crate) mod layer {
    use super::TestLayerDatabaseDescription;

    pub(crate) struct MyInnerInnerData {
        pub(crate) param6: u8,
    }

    impl MyInnerInnerData {
        pub(crate) const fn new() -> Self {
            Self { param6: 0 }
        }
    }

    pub(crate) mod my_inner_inner_data {
        use super::MyInnerInnerData;
        use crate::ToKey;

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Keys {
            Param6,
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Fields {
            Param6(u8),
        }

        impl ToKey<Keys> for Fields {
            fn to_key(&self) -> Keys {
                match self {
                    Fields::Param6(_) => Keys::Param6,
                }
            }
        }

        impl crate::DataFieldAccessor<Keys, Fields> for MyInnerInnerData {
            fn get(&self, key: Keys) -> Fields {
                match key {
                    Keys::Param6 => Fields::Param6(self.param6.clone()),
                }
            }

            fn set(&mut self, field: Fields) {
                match field {
                    Fields::Param6(value) => self.param6 = value,
                }
            }
        }

        impl crate::DataFieldTryAccessor<Keys, u8> for MyInnerInnerData {
            fn try_get(&self, key: Keys) -> Result<u8, crate::AccessorError> {
                match key {
                    Keys::Param6 => Ok(self.param6.clone()),
                }
            }

            fn try_set(&mut self, key: Keys, value: u8) -> Result<(), crate::AccessorError> {
                match key {
                    Keys::Param6 => {
                        self.param6 = value;
                        Ok(())
                    }
                }
            }
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

    pub(crate) mod my_inner_data {
        use super::{MyInnerData, my_inner_inner_data};
        use crate::ToKey;

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Keys {
            Param4,
            Param5,
            Inner3(my_inner_inner_data::Keys),
            Inner4(my_inner_inner_data::Keys),
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Fields {
            Param4(u8),
            Param5(bool),
            Inner3(my_inner_inner_data::Fields),
            Inner4(my_inner_inner_data::Fields),
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum MyInnerInnerDataFolderPath {
            Inner3,
            Inner4,
        }

        impl ToKey<Keys> for Fields {
            fn to_key(&self) -> Keys {
                match self {
                    Fields::Param4(_) => Keys::Param4,
                    Fields::Param5(_) => Keys::Param5,
                    Fields::Inner3(field) => Keys::Inner3(field.to_key()),
                    Fields::Inner4(field) => Keys::Inner4(field.to_key()),
                }
            }
        }

        impl crate::ToFull<Keys, my_inner_inner_data::Keys> for MyInnerInnerDataFolderPath {
            fn build_full(&self, internal: my_inner_inner_data::Keys) -> Keys {
                match self {
                    Self::Inner3 => Keys::Inner3(internal),
                    Self::Inner4 => Keys::Inner3(internal),
                }
            }
        }

        impl crate::ToFull<Fields, my_inner_inner_data::Fields>
            for MyInnerInnerDataFolderPath
        {
            fn build_full(&self, internal: my_inner_inner_data::Fields) -> Fields {
                match self {
                    Self::Inner3 => Fields::Inner3(internal),
                    Self::Inner4 => Fields::Inner3(internal),
                }
            }
        }

        impl crate::DataFieldAccessor<Keys, Fields> for MyInnerData {
            fn get(&self, key: Keys) -> Fields {
                match key {
                    Keys::Param4 => Fields::Param4(self.param4.clone()),
                    Keys::Param5 => Fields::Param5(self.param5.clone()),
                    Keys::Inner3(key) => Fields::Inner3(self.inner3.get(key)),
                    Keys::Inner4(key) => Fields::Inner4(self.inner4.get(key)),
                }
            }

            fn set(&mut self, field: Fields) {
                match field {
                    Fields::Param4(value) => self.param4 = value,
                    Fields::Param5(value) => self.param5 = value,
                    Fields::Inner3(field) => self.inner3.set(field),
                    Fields::Inner4(field) => self.inner4.set(field),
                }
            }
        }

        impl crate::DataFieldTryAccessor<Keys, u8> for MyInnerData {
            fn try_get(&self, key: Keys) -> Result<u8, crate::AccessorError> {
                match key {
                    Keys::Param4 => todo!(),
                    Keys::Inner3(key) => self.inner3.try_get(key),
                    Keys::Inner4(key) => self.inner4.try_get(key),
                    _ => Err(crate::AccessorError::TypeMissmatch("u8")),
                }
            }

            fn try_set(&mut self, key: Keys, value: u8) -> Result<(), crate::AccessorError> {
                match key {
                    Keys::Param4 => {
                        self.param4 = value;
                        Ok(())
                    }
                    Keys::Inner3(key) => self.inner3.try_set(key, value),
                    Keys::Inner4(key) => self.inner4.try_set(key, value),
                    _ => Err(crate::AccessorError::TypeMissmatch("u8")),
                }
            }
        }

        impl crate::DataFieldTryAccessor<Keys, bool> for MyInnerData {
            fn try_get(&self, key: Keys) -> Result<bool, crate::AccessorError> {
                match key {
                    Keys::Param5 => Ok(self.param5.clone()),
                    _ => Err(crate::AccessorError::TypeMissmatch("bool")),
                }
            }

            fn try_set(&mut self, key: Keys, value: bool) -> Result<(), crate::AccessorError> {
                match key {
                    Keys::Param5 => {
                        self.param5 = value;
                        Ok(())
                    }
                    _ => Err(crate::AccessorError::TypeMissmatch("bool")),
                }
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

    pub(crate) mod my_layer_data {
        use super::{MyLayerData, my_inner_data, my_inner_inner_data};

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum AbsKeys {
            Param1,
            Param2,
            Param3,
            Inner1(my_inner_data::Keys),
            Inner2(my_inner_data::Keys),
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        #[allow(dead_code)]
        pub(crate) enum AbsFields {
            Param1(u8),
            Param2(bool),
            Param3(u8),
            Inner1(my_inner_data::Fields),
            Inner2(my_inner_data::Fields),
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
        pub(crate) enum MyInnerInnerDataFolderPath {
            Inner1(my_inner_data::MyInnerInnerDataFolderPath),
            Inner2(my_inner_data::MyInnerInnerDataFolderPath),
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum MyInnerDataFolderPath {
            Inner1,
            Inner2,
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Keys {
            Param1,
            Param2,
            Param3,
            Param4,
            Param5,
            Param6,
        }

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        pub(crate) enum Fields {
            Param1(u8),
            Param2(bool),
            Param3(u8),
            Param4(u8),
            Param5(bool),
            Param6(u8),
        }

        impl crate::ToKey<AbsKeys> for AbsFields {
            fn to_key(&self) -> AbsKeys {
                match self {
                    AbsFields::Param1(_) => AbsKeys::Param1,
                    AbsFields::Param2(_) => AbsKeys::Param2,
                    AbsFields::Param3(_) => AbsKeys::Param3,
                    AbsFields::Inner1(inner) => AbsKeys::Inner1(inner.to_key()),
                    AbsFields::Inner2(inner) => AbsKeys::Inner2(inner.to_key()),
                }
            }
        }

        impl crate::ToKey<Keys> for Fields {
            fn to_key(&self) -> Keys {
                match self {
                    Fields::Param1(_) => Keys::Param1,
                    Fields::Param2(_) => Keys::Param2,
                    Fields::Param3(_) => Keys::Param3,
                    Fields::Param4(_) => Keys::Param4,
                    Fields::Param5(_) => Keys::Param5,
                    Fields::Param6(_) => Keys::Param6,
                }
            }
        }

        impl From<AbsKeys> for Keys {
            fn from(value: AbsKeys) -> Self {
                match value {
                    AbsKeys::Param1 => Keys::Param1,
                    AbsKeys::Param2 => Keys::Param2,
                    AbsKeys::Param3 => Keys::Param3,
                    AbsKeys::Inner1(inner) | AbsKeys::Inner2(inner) => match inner {
                        my_inner_data::Keys::Param4 => Keys::Param4,
                        my_inner_data::Keys::Param5 => Keys::Param5,
                        my_inner_data::Keys::Inner3(inner) | my_inner_data::Keys::Inner4(inner) => {
                            match inner {
                                my_inner_inner_data::Keys::Param6 => Keys::Param6,
                            }
                        }
                    },
                }
            }
        }

        impl crate::ToFromUsize for Keys {
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

        impl From<AbsFields> for Fields {
            fn from(value: AbsFields) -> Self {
                match value {
                    AbsFields::Param1(value) => Fields::Param1(value),
                    AbsFields::Param2(value) => Fields::Param2(value),
                    AbsFields::Param3(value) => Fields::Param3(value),
                    AbsFields::Inner1(inner) | AbsFields::Inner2(inner) => match inner {
                        my_inner_data::Fields::Param4(value) => Fields::Param4(value),
                        my_inner_data::Fields::Param5(value) => Fields::Param5(value),
                        my_inner_data::Fields::Inner3(inner)
                        | my_inner_data::Fields::Inner4(inner) => match inner {
                            my_inner_inner_data::Fields::Param6(value) => Fields::Param6(value),
                        },
                    },
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

        impl crate::ToFull<AbsKeys, my_inner_data::Keys> for MyInnerDataFolderPath {
            fn build_full(&self, internal: my_inner_data::Keys) -> AbsKeys {
                match self {
                    Self::Inner1 => AbsKeys::Inner1(internal),
                    Self::Inner2 => AbsKeys::Inner2(internal),
                }
            }
        }

        impl crate::ToFull<AbsFields, my_inner_data::Fields> for MyInnerDataFolderPath {
            fn build_full(&self, internal: my_inner_data::Fields) -> AbsFields {
                match self {
                    Self::Inner1 => AbsFields::Inner1(internal),
                    Self::Inner2 => AbsFields::Inner2(internal),
                }
            }
        }

        impl crate::ToFull<AbsKeys, my_inner_inner_data::Keys> for MyInnerInnerDataFolderPath {
            fn build_full(&self, internal: my_inner_inner_data::Keys) -> AbsKeys {
                match self {
                    Self::Inner1(inner) => AbsKeys::Inner1(inner.build_full(internal)),
                    Self::Inner2(inner) => AbsKeys::Inner2(inner.build_full(internal)),
                }
            }
        }

        impl crate::ToFull<AbsFields, my_inner_inner_data::Fields>
            for MyInnerInnerDataFolderPath
        {
            fn build_full(&self, internal: my_inner_inner_data::Fields) -> AbsFields {
                match self {
                    Self::Inner1(inner) => AbsFields::Inner1(inner.build_full(internal)),
                    Self::Inner2(inner) => AbsFields::Inner2(inner.build_full(internal)),
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
        pub(crate) const MY_DATA_ABS_KEYS_ALL_VARIANTS: [AbsKeys; MY_DATA_ABS_VARIANT_COUNT] = [
            AbsKeys::Param1,
            AbsKeys::Param2,
            AbsKeys::Param3,
            AbsKeys::Inner1(my_inner_data::Keys::Param4),
            AbsKeys::Inner1(my_inner_data::Keys::Param5),
            AbsKeys::Inner1(my_inner_data::Keys::Inner3(
                my_inner_inner_data::Keys::Param6,
            )),
            AbsKeys::Inner1(my_inner_data::Keys::Inner4(
                my_inner_inner_data::Keys::Param6,
            )),
            AbsKeys::Inner2(my_inner_data::Keys::Param4),
            AbsKeys::Inner2(my_inner_data::Keys::Param5),
            AbsKeys::Inner2(my_inner_data::Keys::Inner3(
                my_inner_inner_data::Keys::Param6,
            )),
            AbsKeys::Inner2(my_inner_data::Keys::Inner4(
                my_inner_inner_data::Keys::Param6,
            )),
        ];

        impl crate::VariantCount for AbsKeys {
            const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
        }

        impl crate::AllVariants for AbsKeys {
            const ALL_VARIANTS: &[Self] = &MY_DATA_ABS_KEYS_ALL_VARIANTS;
        }

        impl crate::VariantCount for AbsFields {
            const COUNT: usize = MY_DATA_ABS_VARIANT_COUNT;
        }

        pub(crate) const MY_DATA_FLAT_VARIANT_COUNT: usize = 6;

        impl crate::VariantCount for Keys {
            const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
        }

        impl crate::VariantCount for Fields {
            const COUNT: usize = MY_DATA_FLAT_VARIANT_COUNT;
        }

        impl crate::AbsKeyConstraints for AbsKeys {}

        impl crate::AbsFieldConstraints<AbsKeys> for AbsFields {}

        impl crate::FlatKeyConstraints<AbsKeys> for Keys {}

        impl crate::FlatFieldConstraints<AbsFields, Keys> for Fields {}

        impl crate::FocusConstraints for MyInnerInnerDataFocus {}
        impl crate::FocusConstraints for MyInnerDataFocus {}

        impl
            crate::PathConstraints<
                (AbsKeys, AbsFields),
                (my_inner_inner_data::Keys, my_inner_inner_data::Fields),
            > for MyInnerInnerDataFolderPath
        {
        }
        impl
            crate::PathConstraints<
                (AbsKeys, AbsFields),
                (my_inner_data::Keys, my_inner_data::Fields),
            > for MyInnerDataFolderPath
        {
        }

        impl crate::DataFieldAccessor<AbsKeys, AbsFields> for MyLayerData {
            fn get(&self, key: AbsKeys) -> AbsFields {
                match key {
                    AbsKeys::Param1 => AbsFields::Param1(self.param1.clone()),
                    AbsKeys::Param2 => AbsFields::Param2(self.param2.clone()),
                    AbsKeys::Param3 => AbsFields::Param3(self.param3.clone()),
                    AbsKeys::Inner1(key) => AbsFields::Inner1(self.inner1.get(key)),
                    AbsKeys::Inner2(key) => AbsFields::Inner2(self.inner2.get(key)),
                }
            }

            fn set(&mut self, field: AbsFields) {
                match field {
                    AbsFields::Param1(value) => self.param1 = value,
                    AbsFields::Param2(value) => self.param2 = value,
                    AbsFields::Param3(value) => self.param3 = value,
                    AbsFields::Inner1(field) => self.inner1.set(field),
                    AbsFields::Inner2(field) => self.inner2.set(field),
                }
            }
        }

        impl crate::DataFieldTryAccessor<AbsKeys, u8> for MyLayerData {
            fn try_get(&self, key: AbsKeys) -> Result<u8, crate::AccessorError> {
                match key {
                    AbsKeys::Param1 => Ok(self.param1.clone()),
                    AbsKeys::Param3 => Ok(self.param3.clone()),
                    AbsKeys::Inner1(key) => self.inner1.try_get(key),
                    AbsKeys::Inner2(key) => self.inner2.try_get(key),
                    _ => Err(crate::AccessorError::TypeMissmatch("u8")),
                }
            }

            fn try_set(&mut self, key: AbsKeys, value: u8) -> Result<(), crate::AccessorError> {
                match key {
                    AbsKeys::Param1 => {
                        self.param1 = value;
                        Ok(())
                    }
                    AbsKeys::Param3 => {
                        self.param3 = value;
                        Ok(())
                    }
                    AbsKeys::Inner1(key) => self.inner1.try_set(key, value),
                    AbsKeys::Inner2(key) => self.inner2.try_set(key, value),
                    _ => Err(crate::AccessorError::TypeMissmatch("u8")),
                }
            }
        }

        impl crate::DataFieldTryAccessor<AbsKeys, bool> for MyLayerData {
            fn try_get(&self, key: AbsKeys) -> Result<bool, crate::AccessorError> {
                match key {
                    AbsKeys::Param2 => Ok(self.param2.clone()),
                    AbsKeys::Inner1(key) => self.inner1.try_get(key),
                    AbsKeys::Inner2(key) => self.inner2.try_get(key),
                    _ => Err(crate::AccessorError::TypeMissmatch("bool")),
                }
            }

            fn try_set(&mut self, key: AbsKeys, value: bool) -> Result<(), crate::AccessorError> {
                match key {
                    AbsKeys::Param2 => {
                        self.param2 = value;
                        Ok(())
                    }
                    AbsKeys::Inner1(key) => self.inner1.try_set(key, value),
                    AbsKeys::Inner2(key) => self.inner2.try_set(key, value),
                    _ => Err(crate::AccessorError::TypeMissmatch("bool")),
                }
            }
        }
    }

    #[allow(unused_imports)]
    pub use my_layer_data::*;
}
