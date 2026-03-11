use crate::DataFieldAccessor;

pub trait VariantCount {
    const COUNT: usize;
}

pub trait AllVariants
where
    Self: VariantCount + Sized + 'static,
{
    const ALL_VARIANTS: &[Self];
}

pub trait ToKey<Key> {
    fn to_key(&self) -> Key;
}

pub trait ToFromUsize
where
    Self: Sized,
{
    fn to_usize(&self) -> usize;
    fn try_from_usize(value: usize) -> Option<Self>;
}

pub trait AbsKeyConstraints: Eq + Clone + Copy + Ord + AllVariants {}
pub trait AbsFieldConstraints<AbsKey>: Eq + Clone + VariantCount + ToKey<AbsKey> {}

pub trait AbsFolderConstraints: Eq + Clone + Copy + Ord + VariantCount {}

pub trait FlatKeyConstraints<AbsKey>:
    Eq + Clone + Copy + Ord + From<AbsKey> + VariantCount + ToFromUsize
{
}
pub trait FlatFieldConstraints<AbsField, FlatKey>:
    Eq + Clone + VariantCount + From<AbsField> + VariantCount + ToKey<FlatKey>
{
}

pub trait FlatFolderConstraints<AbsFolder>:
    Eq + Clone + Copy + Ord + From<AbsFolder> + VariantCount
{
}

pub trait DatabaseDescription {
    type AbsKey: AbsKeyConstraints;
    type AbsField: AbsFieldConstraints<Self::AbsKey>;
    type AbsFolder: AbsFolderConstraints;

    type FlatKey: FlatKeyConstraints<Self::AbsKey>;
    type FlatField: FlatFieldConstraints<Self::AbsField, Self::FlatKey>;
    type FlatFolder: FlatFolderConstraints<Self::AbsFolder>;

    type Data: DataFieldAccessor<Self::AbsKey, Self::AbsField>;
}

pub trait FlatDatabaseDescription {
    type Key: AbsKeyConstraints + FlatKeyConstraints<Self::Key>;
    type Field: AbsFieldConstraints<Self::Key> + FlatFieldConstraints<Self::Field, Self::Key>;
    type Data: DataFieldAccessor<Self::Key, Self::Field>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DummyFolder {}

impl VariantCount for DummyFolder {
    const COUNT: usize = 0;
}

impl AbsFolderConstraints for DummyFolder {}
impl FlatFolderConstraints<DummyFolder> for DummyFolder {}

impl<Database: FlatDatabaseDescription> DatabaseDescription for Database {
    type AbsKey = Database::Key;
    type AbsField = Database::Field;
    type AbsFolder = DummyFolder;
    type FlatKey = Database::Key;
    type FlatField = Database::Field;
    type FlatFolder = DummyFolder;
    type Data = Database::Data;
}
