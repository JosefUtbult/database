use crate::{
    Subset,
    content::DatabaseContent,
    database::{DatabaseRef, ParameterChangeList},
};

/// A `DatabaseSubscriber` is any entity that needs to subscribe to a subset of parameters in a
/// database. This subset is decided by the `ParameterSubset`. A parameter subset is a struct that
/// is registered with the `Database` as one permutation of variables present in the database
pub trait DatabaseSubscriber<ParameterSubset, Parameter, const PARAMETER_COUNT: usize>
where
    Parameter: Clone + Copy + Eq,
    ParameterSubset: Subset<Parameter, PARAMETER_COUNT> + Clone + Copy,
{
    fn on_set(&self, change: &ParameterSubset);
}

/// A `DatabaseSubscriberHandler` is an handler that is built automatically using the `Database`
/// proc-macro. This handler will go through a list of parameters and notify all subscribers
/// relevant to the changes
pub trait DatabaseSubscriberHandler<InternalContent, Parameter, const PARAMETER_COUNT: usize>
where
    Parameter: Clone + Copy + Eq,
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
{
    fn notify_subscribers(
        &self,
        database: &dyn DatabaseRef<Parameter>,
        parameter_change: &ParameterChangeList<Parameter, PARAMETER_COUNT>,
    );
}
