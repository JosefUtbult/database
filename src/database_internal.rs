use core::marker::PhantomData;

use crate::DataFieldAccessor;

pub(crate) struct InternalMutable<Data, AbsKey, AbsField>
where
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) data: Data,
    pub(crate) _abs_key: PhantomData<AbsKey>,
    pub(crate) _abs_fields: PhantomData<AbsField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    LockFail,
    SubscriberOverflow,
}

impl<Data, AbsKey, AbsField> InternalMutable<Data, AbsKey, AbsField>
where
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub const fn new(data: Data) -> Self {
        Self {
            data,
            _abs_key: PhantomData,
            _abs_fields: PhantomData,
        }
    }
}
