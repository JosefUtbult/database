use crate::database_core::{AllVariants, VariantCount};

pub trait AbsKeyConstraints<AbsField>:
    Eq + Clone + Copy + Ord + AllVariants + From<AbsField>
{
}
pub trait AbsFieldConstraints: Eq + Clone + Copy + VariantCount {}

pub trait FlatKeyConstraints<AbsKey, FlatField>:
    Eq + Clone + Copy + Ord + From<FlatField> + From<AbsKey>
{
}
pub trait FlatFieldConstraints<AbsField>: Eq + Clone + Copy + VariantCount + From<AbsField> {}

pub trait UsizeConstraints<FlatKey>: From<FlatKey> {}
