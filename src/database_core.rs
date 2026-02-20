use core::{hash::Hash, marker::PhantomData};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, SubscriberData,
    database_internal::{DatabaseError, InternalMutable},
    mutex::ScopedLocked,
};

pub trait AllKeys<const ABS_PARAMETER_COUNT: usize>
where
    Self: Sized,
{
    const ALL_KEYS: [Self; ABS_PARAMETER_COUNT];
}

pub(crate) struct DatabaseCore<
    'a,
    Mutex,
    Data,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> where
    Mutex: ScopedRawMutex + ConstInit,
    AbsKey: AllKeys<ABS_PARAMETER_COUNT>,
    FlatKey: Ord + Hash + Copy + From<AbsKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) data: ScopedLocked<Mutex, InternalMutable<Data, AbsKey, AbsField>>,
    pub(crate) subscribers: SubscriberData<'a, Mutex, FlatKey, FLAT_PARAMETER_COUNT>,
    _fields: PhantomData<FlatField>,
}

impl<
    'a,
    Mutex,
    Data,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
>
    DatabaseCore<
        'a,
        Mutex,
        Data,
        AbsKey,
        AbsField,
        FlatKey,
        FlatField,
        ABS_PARAMETER_COUNT,
        FLAT_PARAMETER_COUNT,
    >
where
    Mutex: ScopedRawMutex + ConstInit,
    AbsKey: AllKeys<ABS_PARAMETER_COUNT>,
    AbsKey: From<AbsField> + Copy,
    AbsField: Eq + PartialEq + Copy,
    FlatKey: Ord + Hash + Copy + From<FlatField> + From<AbsKey>,
    FlatField: Copy + Eq + PartialEq + From<AbsField>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) const fn new(data: Data) -> Self {
        Self {
            data: ScopedLocked::new(InternalMutable::new(data)),
            subscribers: SubscriberData::new(),
            _fields: PhantomData,
        }
    }

    pub(crate) fn get(&self, key: AbsKey) -> Result<FlatField, DatabaseError> {
        match self.data.try_with(|internal| {
            let field = internal.data.get(key);
            field.into()
        }) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub(crate) fn set(&self, flat_key: FlatKey, abs_field: AbsField) -> Result<(), DatabaseError> {
        #[derive(PartialEq, Eq)]
        enum SetState {
            Updated,
            UpToDate,
        }

        let abs_key: AbsKey = abs_field.into();

        match self.data.try_with(|internal| {
            if internal.data.get(abs_key) != abs_field {
                internal.data.set(abs_field);
                SetState::Updated
            } else {
                SetState::UpToDate
            }
        }) {
            None => Err(DatabaseError::LockFail),
            Some(set_state) => {
                if set_state == SetState::Updated {
                    self.subscribers.on_change(flat_key);
                }
                Ok(())
            }
        }
    }
}
