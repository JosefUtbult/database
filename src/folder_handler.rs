use crate::{DynamicKeySet, PathDescription, database_core::DatabaseError};

pub trait FolderHandler<Path, Description: PathDescription> {
    type Content;

    fn get_at<'a>(
        &'a self,
        path: Path
    ) -> &'a Self::Content;

    fn get_at_mut<'a>(
        &'a mut self,
        path: Path
    ) -> &'a mut Self::Content;

    fn compare(
        &self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content
    ) -> Result<(), DatabaseError>;

    fn clone(
        &mut self,
        differing_keys: &mut dyn DynamicKeySet<Description::AbsKey, Description::FlatKey>,
        path: Path,
        other: &Self::Content
    ) -> Result<(), DatabaseError>;
}
