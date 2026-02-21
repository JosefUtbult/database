pub trait VariantCount {
    const COUNT: usize;
}

pub trait AllVariants
where
    Self: VariantCount + Sized + 'static,
{
    const ALL_VARIANTS: &[Self];
}

pub trait AbsKeyConstraints<AbsField>:
    Eq + Clone + Copy + Ord + AllVariants + From<AbsField>
{
}
pub trait AbsFieldConstraints: Eq + Clone + VariantCount {}

pub trait FlatKeyConstraints<AbsKey, FlatField>:
    Eq + Clone + Copy + Ord + From<FlatField> + From<AbsKey> + VariantCount
{
}
pub trait FlatFieldConstraints<AbsField>:
    Eq + Clone + VariantCount + From<AbsField> + VariantCount
{
}

pub trait UsizeConstraints<FlatKey>: From<FlatKey> {}
