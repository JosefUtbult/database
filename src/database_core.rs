use core::marker::PhantomData;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, SubscriberData,
    database_internal::{DatabaseError, InternalMutable},
    database_traits::{
        AbsFieldConstraints, AbsKeyConstraints, FlatFieldConstraints, FlatKeyConstraints,
        UsizeConstraints,
    },
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
    AbsKey: AbsKeyConstraints<AbsField, ABS_PARAMETER_COUNT>,
    AbsField: AbsFieldConstraints,
    FlatKey: FlatKeyConstraints<AbsKey, FlatField>,
    FlatField: FlatFieldConstraints<AbsField>,
    usize: UsizeConstraints<FlatKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) data: ScopedLocked<
        Mutex,
        InternalMutable<
            Data,
            AbsKey,
            AbsField,
            FlatKey,
            FlatField,
            ABS_PARAMETER_COUNT,
            FLAT_PARAMETER_COUNT,
        >,
    >,
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
    AbsKey: AbsKeyConstraints<AbsField, ABS_PARAMETER_COUNT>,
    AbsField: AbsFieldConstraints,
    FlatKey: FlatKeyConstraints<AbsKey, FlatField>,
    FlatField: FlatFieldConstraints<AbsField>,
    usize: UsizeConstraints<FlatKey>,
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

    pub fn clone(&self, other: &Self) -> Result<(), DatabaseError> {
        match self.data.try_with(|internal| {
            other
                .data
                .try_with(|other_internal| internal.clone(&other_internal))
        }) {
            None => Err(DatabaseError::LockFail),
            Some(changes) => match changes {
                None => Err(DatabaseError::LockFail),
                Some(_changes) => Ok(()),
            },
        }
    }
}
