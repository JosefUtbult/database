use crate::{DynamicKeySet, PathDescription, database_core::DatabaseError};

pub trait FolderHandler<Path, Description: PathDescription> {
    type Content;

    fn get_at<'a>(&'a self, path: Path) -> &'a Self::Content;

    fn get_at_mut<'a>(&'a mut self, path: Path) -> &'a mut Self::Content;

    fn compare(
        &self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content,
    ) -> Result<(), DatabaseError>;

    fn clone(
        &mut self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content,
    ) -> Result<(), DatabaseError>;
}

#[cfg(test)]
pub(crate) mod test_data_folder_handler {
    use crate::{
        DatabaseError, DynamicKeySet, FolderHandler,
        test_types::{
            TestLayerDatabaseDescription,
            layer::{AbsKeys, Keys, MyInnerData, MyInnerDataFolderPath, MyLayerData},
        },
    };

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

        fn compare(
            &self,
            _differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            _path: MyInnerDataFolderPath,
            _other: &Self::Content,
        ) -> Result<(), DatabaseError> {
            todo!()
        }

        fn clone(
            &mut self,
            _differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            _path: MyInnerDataFolderPath,
            _other: &Self::Content,
        ) -> Result<(), crate::DatabaseError> {
            todo!()
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
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            _path: (),
            other: &Self::Content,
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

            Ok(())
        }

        fn clone(
            &mut self,
            differing_keys: &mut dyn DynamicKeySet<AbsKeys, Keys>,
            _path: (),
            other: &Self::Content,
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

            Ok(())
        }
    }
}
