use crate::{DatabaseRef, ParameterChangeList};

pub trait Subset<Parameter, const PARAMETER_COUNT: usize>: Clone + Copy
where
    Parameter: Clone + Copy + Eq,
{
    /// Check if the parameter change list contains any changes relevant to the subset
    fn is_subscribed(parameter_change: &ParameterChangeList<Parameter, PARAMETER_COUNT>) -> bool;

    fn build_from_database(database: &dyn DatabaseRef<Parameter>) -> Self;
}
