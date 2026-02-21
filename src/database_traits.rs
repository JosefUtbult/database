use crate::database_core::AllKeys;

pub trait BaseConstraint: Eq + Clone + Copy {}

pub trait AbsKeyConstraints<AbsField, const ABS_PARAMETER_COUNT: usize>:
    Eq + Clone + Copy + Ord + AllKeys<ABS_PARAMETER_COUNT> + From<AbsField>
{
}
pub trait AbsFieldConstraints: Eq + Clone + Copy {}

pub trait FlatKeyConstraints<AbsKey, FlatField>:
    Eq + Clone + Copy + Ord + From<FlatField> + From<AbsKey>
{
}
pub trait FlatFieldConstraints<AbsField>: Eq + Clone + Copy + From<AbsField> {}

pub trait UsizeConstraints<FlatKey>: From<FlatKey> {}
