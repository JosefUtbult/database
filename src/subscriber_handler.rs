use crate::{
    content::DatabaseContent,
    database::{DatabaseRef, ParameterChangeList},
};

pub trait DatabaseSubscriber<DataType>
where
    DataType: Clone + Copy,
{
    fn on_set(&self, change: &DataType);
}

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
