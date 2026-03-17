use crate::{DataFieldAccessor, FolderHandler};

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

pub trait Pair {
    type Key;
    type Field;
}

impl<Key, Field> Pair for (Key, Field) {
    type Key = Key;
    type Field = Field;
}

pub trait AbsPair {
    type Key: AbsKeyConstraints;
    type Field: AbsFieldConstraints<Self::Key>;
}

impl<Key, Field> AbsPair for (Key, Field)
where
    Key: AbsKeyConstraints,
    Field: AbsFieldConstraints<Key>,
{
    type Key = Key;
    type Field = Field;
}

pub trait FlatKeyConstraints<AbsKey>:
    Eq + Clone + Copy + Ord + From<AbsKey> + VariantCount + ToFromUsize
{
}

pub trait FlatFieldConstraints<AbsField, FlatKey>:
    Eq + Clone + VariantCount + From<AbsField> + VariantCount + ToKey<FlatKey>
{
}

pub trait FlatPair<Abs: AbsPair> {
    type Key: FlatKeyConstraints<Abs::Key>;
    type Field: FlatFieldConstraints<Abs::Field, Self::Key>;
}

impl<Key, Field, Abs: AbsPair> FlatPair<Abs> for (Key, Field)
where
    Key: FlatKeyConstraints<Abs::Key>,
    Field: FlatFieldConstraints<Abs::Field, Key>,
{
    type Key = Key;
    type Field = Field;
}

pub trait AbsFlatPair {
    type Key: AbsKeyConstraints + FlatKeyConstraints<Self::Key>;
    type Field: AbsFieldConstraints<Self::Key> + FlatFieldConstraints<Self::Field, Self::Key>;
}

impl<Key, Field> AbsFlatPair for (Key, Field)
where
    Key: AbsKeyConstraints + FlatKeyConstraints<Key>,
    Field: AbsFieldConstraints<Key> + FlatFieldConstraints<Field, Key>,
{
    type Key = Key;
    type Field = Field;
}

pub trait PathDescription {
    type AbsKey: AbsKeyConstraints;
    type AbsField: AbsFieldConstraints<Self::AbsKey>;

    type FlatKey: FlatKeyConstraints<Self::AbsKey>;
    type FlatField: FlatFieldConstraints<Self::AbsField, Self::FlatKey>;
}

pub trait DatabaseDescription: Sized {
    type AbsKey: AbsKeyConstraints;
    type AbsField: AbsFieldConstraints<Self::AbsKey>;

    type FlatKey: FlatKeyConstraints<Self::AbsKey>;
    type FlatField: FlatFieldConstraints<Self::AbsField, Self::FlatKey>;

    type Data: DataFieldAccessor<Self::AbsKey, Self::AbsField> + FolderHandler<(), Self>;
}

impl<Abs, Flat, Data> DatabaseDescription for (Abs, Flat, Data)
where
    Abs: AbsPair,
    Flat: FlatPair<Abs>,
    Data: DataFieldAccessor<Abs::Key, Abs::Field> + FolderHandler<(), Self>,
{
    type AbsKey = Abs::Key;
    type AbsField = Abs::Field;
    type FlatKey = Flat::Key;
    type FlatField = Flat::Field;
    type Data = Data;
}

impl<Description: DatabaseDescription> PathDescription for Description {
    type AbsKey = <Self as DatabaseDescription>::AbsKey;
    type AbsField = <Self as DatabaseDescription>::AbsField;
    type FlatKey = <Self as DatabaseDescription>::FlatKey;
    type FlatField = <Self as DatabaseDescription>::FlatField;
}

pub trait FlatDatabaseDescription: Sized {
    type Key: AbsKeyConstraints + FlatKeyConstraints<Self::Key>;
    type Field: AbsFieldConstraints<Self::Key> + FlatFieldConstraints<Self::Field, Self::Key>;
    type Data: DataFieldAccessor<Self::Key, Self::Field> + FolderHandler<(), Self>;
}

impl<Database: FlatDatabaseDescription> DatabaseDescription for Database {
    type AbsKey = Database::Key;
    type AbsField = Database::Field;
    type FlatKey = Database::Key;
    type FlatField = Database::Field;
    type Data = Database::Data;
}

impl<Pair, Data> FlatDatabaseDescription for (Pair, Data)
where
    Pair: AbsFlatPair,
    Data: DataFieldAccessor<Pair::Key, Pair::Field> + FolderHandler<(), Self>,
{
    type Key = Pair::Key;
    type Field = Pair::Field;
    type Data = Data;
}
