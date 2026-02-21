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

pub trait AbsKeyConstraints:
    Eq + Clone + Copy + Ord + AllVariants
{
}
pub trait AbsFieldConstraints<AbsKey>: Eq + Clone + VariantCount + ToKey<AbsKey> {}

pub trait FlatKeyConstraints<AbsKey>:
    Eq + Clone + Copy + Ord + From<AbsKey> + VariantCount
{
}
pub trait FlatFieldConstraints<AbsField, FlatKey>:
    Eq + Clone + VariantCount + From<AbsField> + VariantCount + ToKey<FlatKey>
{
}

pub trait UsizeConstraints<FlatKey>: From<FlatKey> {}
