use crate::{DynamicKeySet, PathDescription, database_core::DatabaseError};

pub trait Folder<Description: PathDescription> {
    fn compare(
        &self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        other: &Self,
    ) -> Result<(), DatabaseError>;

    fn clone(
        &mut self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        other: &Self,
    ) -> Result<(), DatabaseError>;
}

pub trait FolderHandler<Path, Description: PathDescription> {
    type Content;

    fn get_at<'a>(&'a self, path: Path) -> &'a Self::Content;

    fn get_at_mut<'a>(&'a mut self, path: Path) -> &'a mut Self::Content;

    fn compare_path(
        &self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content,
    ) -> Result<(), DatabaseError>;

    fn clone_path(
        &mut self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content,
    ) -> Result<(), DatabaseError>;
}

#[cfg(test)]
pub(crate) mod test_data_folder_handler {
    use core::ptr;

    use crate::{
        DatabaseError, DynamicKeySet, Folder, FolderHandler, KeySet,
        test_types::{
            TestLayerDatabaseDescription,
            layer::{
                AbsKeys, Keys, MY_DATA_FLAT_VARIANT_COUNT, MyInnerData, MyInnerDataFolderPath,
                MyInnerInnerData, MyInnerInnerDataFolderPath, MyLayerData, my_inner_data,
            },
        },
    };

    impl Folder<TestLayerDatabaseDescription> for MyInnerInnerData {
        fn compare(
            &self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param6 != other.param6 {
                differing_keys.insert_flat_key(Keys::Param6)?;
            }
            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param6 != other.param6 {
                self.param6 = other.param6;
                differing_keys.insert_flat_key(Keys::Param6)?;
            }
            Ok(())
        }
    }

    impl Folder<TestLayerDatabaseDescription> for MyInnerData {
        fn compare(
            &self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param4 != other.param4 {
                differing_keys.insert_flat_key(Keys::Param4)?;
            }

            if self.param5 != other.param5 {
                differing_keys.insert_flat_key(Keys::Param5)?;
            }

            self.inner3.compare(differing_keys, &other.inner3)?;
            self.inner4.compare(differing_keys, &other.inner4)?;

            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param4 != other.param4 {
                self.param4 = other.param4;
                differing_keys.insert_flat_key(Keys::Param4)?;
            }

            if self.param5 != other.param5 {
                self.param5 = other.param5;
                differing_keys.insert_flat_key(Keys::Param5)?;
            }

            self.inner3.clone(differing_keys, &other.inner3)?;
            self.inner4.clone(differing_keys, &other.inner4)?;

            Ok(())
        }
    }

    impl Folder<TestLayerDatabaseDescription> for MyLayerData {
        fn compare(
            &self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param1 != other.param1 {
                differing_keys.insert_flat_key(Keys::Param1)?;
            }

            if self.param2 != other.param2 {
                differing_keys.insert_flat_key(Keys::Param2)?;
            }

            if self.param3 != other.param3 {
                differing_keys.insert_flat_key(Keys::Param3)?;
            }

            self.inner1.compare(differing_keys, &other.inner1)?;
            self.inner2.compare(differing_keys, &other.inner2)?;

            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            other: &Self,
        ) -> Result<(), DatabaseError> {
            if self.param1 != other.param1 {
                self.param1 = other.param1;
                differing_keys.insert_flat_key(Keys::Param1)?;
            }

            if self.param2 != other.param2 {
                self.param2 = other.param2;
                differing_keys.insert_flat_key(Keys::Param2)?;
            }

            if self.param3 != other.param3 {
                self.param3 = other.param3;
                differing_keys.insert_flat_key(Keys::Param3)?;
            }

            self.inner1.clone(differing_keys, &other.inner1)?;
            self.inner2.clone(differing_keys, &other.inner2)?;

            Ok(())
        }
    }

    impl FolderHandler<my_inner_data::MyInnerInnerDataFolderPath, TestLayerDatabaseDescription>
        for MyInnerData
    {
        type Content = MyInnerInnerData;

        fn get_at<'a>(
            &'a self,
            path: my_inner_data::MyInnerInnerDataFolderPath,
        ) -> &'a Self::Content {
            match path {
                my_inner_data::MyInnerInnerDataFolderPath::Inner3 => &self.inner3,
                my_inner_data::MyInnerInnerDataFolderPath::Inner4 => &self.inner4,
            }
        }

        fn get_at_mut<'a>(
            &'a mut self,
            path: my_inner_data::MyInnerInnerDataFolderPath,
        ) -> &'a mut Self::Content {
            match path {
                my_inner_data::MyInnerInnerDataFolderPath::Inner3 => &mut self.inner3,
                my_inner_data::MyInnerInnerDataFolderPath::Inner4 => &mut self.inner4,
            }
        }

        fn compare_path(
            &self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            path: my_inner_data::MyInnerInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            match path {
                my_inner_data::MyInnerInnerDataFolderPath::Inner3 => {
                    self.inner3.compare(differing_keys, other)
                }
                my_inner_data::MyInnerInnerDataFolderPath::Inner4 => {
                    self.inner4.compare(differing_keys, other)
                }
            }
        }

        fn clone_path(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<
                <TestLayerDatabaseDescription as crate::PathDescription>::AbsKey,
                <TestLayerDatabaseDescription as crate::PathDescription>::FlatKey,
            >,
            path: my_inner_data::MyInnerInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            match path {
                my_inner_data::MyInnerInnerDataFolderPath::Inner3 => {
                    self.inner3.clone(differing_keys, other)
                }
                my_inner_data::MyInnerInnerDataFolderPath::Inner4 => {
                    self.inner4.clone(differing_keys, other)
                }
            }
        }
    }

    impl FolderHandler<MyInnerInnerDataFolderPath, TestLayerDatabaseDescription> for MyLayerData {
        type Content = MyInnerInnerData;

        fn get_at<'a>(&'a self, path: MyInnerInnerDataFolderPath) -> &'a Self::Content {
            match path {
                MyInnerInnerDataFolderPath::Inner1(folder_path) => self.inner1.get_at(folder_path),
                MyInnerInnerDataFolderPath::Inner2(folder_path) => self.inner2.get_at(folder_path),
            }
        }

        fn get_at_mut<'a>(&'a mut self, path: MyInnerInnerDataFolderPath) -> &'a mut Self::Content {
            match path {
                MyInnerInnerDataFolderPath::Inner1(folder_path) => {
                    self.inner1.get_at_mut(folder_path)
                }
                MyInnerInnerDataFolderPath::Inner2(folder_path) => {
                    self.inner2.get_at_mut(folder_path)
                }
            }
        }

        fn compare_path(
            &self,
            differing_keys: &mut dyn DynamicKeySet<
                <TestLayerDatabaseDescription as crate::PathDescription>::AbsKey,
                <TestLayerDatabaseDescription as crate::PathDescription>::FlatKey,
            >,
            path: MyInnerInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            match path {
                MyInnerInnerDataFolderPath::Inner1(folder_path) => {
                    self.inner1.compare_path(differing_keys, folder_path, other)
                }
                MyInnerInnerDataFolderPath::Inner2(folder_path) => {
                    self.inner2.compare_path(differing_keys, folder_path, other)
                }
            }
        }

        fn clone_path(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<
                <TestLayerDatabaseDescription as crate::PathDescription>::AbsKey,
                <TestLayerDatabaseDescription as crate::PathDescription>::FlatKey,
            >,
            path: MyInnerInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            match path {
                MyInnerInnerDataFolderPath::Inner1(folder_path) => {
                    self.inner1.clone_path(differing_keys, folder_path, other)
                }
                MyInnerInnerDataFolderPath::Inner2(folder_path) => {
                    self.inner2.clone_path(differing_keys, folder_path, other)
                }
            }
        }
    }

    impl FolderHandler<MyInnerDataFolderPath, TestLayerDatabaseDescription> for MyLayerData {
        type Content = MyInnerData;

        fn get_at<'a>(&'a self, path: MyInnerDataFolderPath) -> &'a Self::Content {
            match path {
                MyInnerDataFolderPath::Inner1 => &self.inner1,
                MyInnerDataFolderPath::Inner2 => &self.inner2,
            }
        }

        fn get_at_mut<'a>(&'a mut self, path: MyInnerDataFolderPath) -> &'a mut Self::Content {
            match path {
                MyInnerDataFolderPath::Inner1 => &mut self.inner1,
                MyInnerDataFolderPath::Inner2 => &mut self.inner2,
            }
        }

        fn compare_path(
            &self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            path: MyInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            match path {
                MyInnerDataFolderPath::Inner1 => self.inner1.compare(differing_keys, other),
                MyInnerDataFolderPath::Inner2 => self.inner2.compare(differing_keys, other),
            }
        }

        fn clone_path(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            path: MyInnerDataFolderPath,
            other: &Self::Content,
        ) -> Result<(), crate::DatabaseError> {
            match path {
                MyInnerDataFolderPath::Inner1 => self.inner1.clone(differing_keys, other),
                MyInnerDataFolderPath::Inner2 => self.inner2.clone(differing_keys, other),
            }
        }
    }

    fn create_key_set() -> KeySet<AbsKeys, Keys, MY_DATA_FLAT_VARIANT_COUNT> {
        KeySet::new()
    }

    #[test]
    fn compare_inner_inner_data() {
        let data1 = MyInnerInnerData::new();
        let mut data2 = MyInnerInnerData::new();
        data2.param6 = 3;

        let mut differing_keys = create_key_set();
        data1.compare(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 1);
        assert!(differing_keys.contains(&Keys::Param6));
    }

    #[test]
    fn compare_inner_data() {
        let data1 = MyInnerData::new();
        let mut data2 = MyInnerData::new();
        data2.param4 = 3;
        data2.inner4.param6 = 12;

        let mut differing_keys = create_key_set();
        data1.compare(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 2);
        assert!(differing_keys.contains(&Keys::Param4));
        assert!(differing_keys.contains(&Keys::Param6));
    }

    #[test]
    fn compare_layer_data() {
        let data1 = MyLayerData::new();
        let mut data2 = MyLayerData::new();
        data2.param1 = 3;
        data2.inner1.param5 = true;
        data2.inner1.inner4.param6 = 12;
        data2.inner2.inner4.param6 = 13;

        let mut differing_keys = create_key_set();
        data1.compare(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 3);
        assert!(differing_keys.contains(&Keys::Param1));
        assert!(differing_keys.contains(&Keys::Param5));
        assert!(differing_keys.contains(&Keys::Param6));
    }

    #[test]
    fn clone_inner_inner_data() {
        let mut data1 = MyInnerInnerData::new();
        let mut data2 = MyInnerInnerData::new();
        data2.param6 = 3;

        let mut differing_keys = create_key_set();
        data1.clone(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 1);
        assert!(differing_keys.contains(&Keys::Param6));
        assert_eq!(data1.param6, 3);
    }

    #[test]
    fn clone_inner_data() {
        let mut data1 = MyInnerData::new();
        let mut data2 = MyInnerData::new();
        data2.param4 = 3;
        data2.inner4.param6 = 12;

        let mut differing_keys = create_key_set();
        data1.clone(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 2);
        assert!(differing_keys.contains(&Keys::Param4));
        assert!(differing_keys.contains(&Keys::Param6));

        assert_eq!(data1.param4, 3);
        assert_eq!(data1.inner4.param6, 12);
    }

    #[test]
    fn clone_layer_data() {
        let mut data1 = MyLayerData::new();
        let mut data2 = MyLayerData::new();
        data2.param1 = 3;
        data2.inner1.param5 = true;
        data2.inner1.inner4.param6 = 12;
        data2.inner2.inner4.param6 = 13;

        let mut differing_keys = create_key_set();
        data1.clone(&mut differing_keys, &data2).unwrap();

        let differing_keys = differing_keys.to_vector();
        assert_eq!(differing_keys.len(), 3);
        assert!(differing_keys.contains(&Keys::Param1));
        assert!(differing_keys.contains(&Keys::Param5));
        assert!(differing_keys.contains(&Keys::Param6));

        assert_eq!(data1.param1, 3);
        assert_eq!(data1.inner1.param5, true);
        assert_eq!(data1.inner1.inner4.param6, 12);
        assert_eq!(data1.inner2.inner4.param6, 13);
    }

    #[test]
    fn get_folder_at() {
        let data = MyLayerData::new();
        let _ = data.get_at(MyInnerInnerDataFolderPath::Inner1(
            my_inner_data::MyInnerInnerDataFolderPath::Inner3,
        ));
    }

    #[test]
    fn get_folder_at_mut() {
        let mut data = MyLayerData::new();
        let _ = data.get_at_mut(MyInnerInnerDataFolderPath::Inner1(
            my_inner_data::MyInnerInnerDataFolderPath::Inner3,
        ));
    }

    #[test]
    fn get_folder_at_returns_correct() {
        let data = MyLayerData::new();
        let result = data.get_at(MyInnerInnerDataFolderPath::Inner1(
            my_inner_data::MyInnerInnerDataFolderPath::Inner3,
        ));

        assert!(ptr::eq(result, &data.inner1.inner3));
        assert!(!ptr::eq(result, &data.inner1.inner4));
        assert!(!ptr::eq(result, &data.inner2.inner3));
    }

    #[test]
    fn compare_path() {
        let data1 = MyLayerData::new();
        let mut data2 = MyInnerInnerData::new();
        data2.param6 = 12;

        let mut differing_keys = create_key_set();
        data1
            .compare_path(
                &mut differing_keys,
                MyInnerInnerDataFolderPath::Inner1(
                    my_inner_data::MyInnerInnerDataFolderPath::Inner3,
                ),
                &data2,
            )
            .unwrap();

        let differing_keys = differing_keys.to_vector();

        assert_eq!(differing_keys.len(), 1);
        assert!(differing_keys.contains(&Keys::Param6));
    }

    #[test]
    fn clone_path() {
        let mut data1 = MyLayerData::new();
        let mut data2 = MyInnerInnerData::new();
        data2.param6 = 12;

        let mut differing_keys = create_key_set();
        data1
            .clone_path(
                &mut differing_keys,
                MyInnerInnerDataFolderPath::Inner1(
                    my_inner_data::MyInnerInnerDataFolderPath::Inner3,
                ),
                &data2,
            )
            .unwrap();

        let differing_keys = differing_keys.to_vector();

        assert_eq!(differing_keys.len(), 1);
        assert!(differing_keys.contains(&Keys::Param6));
        assert_eq!(data1.inner1.inner3.param6, 12);
    }
}
