use core::marker::PhantomData;
use heapless::Vec;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, FocusHandler, SubscriberData,
    database_traits::{
        AbsFieldConstraints, AbsKeyConstraints, FlatFieldConstraints, FlatKeyConstraints,
        UsizeConstraints,
    },
    mutex::ScopedLocked,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    LockFail,
    SubscriberOverflow,
    ParameterCountMissmatch,
}

struct KeySet<Key, const KEY_COUNT: usize>([Option<Key>; KEY_COUNT]);

impl<Key, const KEY_COUNT: usize> KeySet<Key, KEY_COUNT>
where
    Key: Eq + Copy,
    usize: From<Key>,
{
    const fn new() -> Self {
        Self([const { None }; KEY_COUNT])
    }

    fn insert(&mut self, key: Key) -> Result<(), DatabaseError> {
        let index: usize = key.clone().into();
        if index < KEY_COUNT {
            if self.0[index].is_none() {
                let _ = self.0[index].insert(key);
            }
            Ok(())
        } else {
            Err(DatabaseError::ParameterCountMissmatch)
        }
    }

    fn get_vector(&mut self) -> Vec<Key, KEY_COUNT> {
        let mut result = Vec::new();
        for element in self.0.iter_mut() {
            if let Some(element) = element.take() {
                if result.push(element).is_err() {
                    panic!("Vector overflow");
                }
            }
        }
        result
    }
}

pub(crate) struct InternalMutable<
    Data,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> where
    AbsKey: AbsKeyConstraints<AbsField>,
    AbsField: AbsFieldConstraints,
    FlatKey: FlatKeyConstraints<AbsKey, FlatField>,
    FlatField: FlatFieldConstraints<AbsField>,
    usize: UsizeConstraints<FlatKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) data: Data,
    pub(crate) _abs_key: PhantomData<AbsKey>,
    pub(crate) _abs_fields: PhantomData<AbsField>,
    pub(crate) _flat_key: PhantomData<FlatKey>,
    pub(crate) _flat_fields: PhantomData<FlatField>,
}

impl<
    Data,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
>
    InternalMutable<
        Data,
        AbsKey,
        AbsField,
        FlatKey,
        FlatField,
        ABS_PARAMETER_COUNT,
        FLAT_PARAMETER_COUNT,
    >
where
    AbsKey: AbsKeyConstraints<AbsField>,
    AbsField: AbsFieldConstraints,
    FlatKey: FlatKeyConstraints<AbsKey, FlatField>,
    FlatField: FlatFieldConstraints<AbsField>,
    usize: UsizeConstraints<FlatKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    pub(crate) const fn new(data: Data) -> Self {
        Self {
            data,
            _abs_key: PhantomData,
            _abs_fields: PhantomData,
            _flat_key: PhantomData,
            _flat_fields: PhantomData,
        }
    }

    pub(crate) fn clone<Focus>(
        &mut self,
        other: &Self,
        focus_handler: &Focus,
    ) -> Vec<FlatKey, FLAT_PARAMETER_COUNT>
    where
        Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
    {
        let mut changed_key_set: KeySet<FlatKey, FLAT_PARAMETER_COUNT> = KeySet::new();
        for abs_key in AbsKey::ALL_VARIANTS.iter() {
            let this_abs_field = self.data.get(*abs_key);
            let other_abs_field = other.data.get(*abs_key);

            if this_abs_field != other_abs_field {
                self.data.set(other_abs_field);
                let focused_abs_key = focus_handler.get_focus_key((*abs_key).into());
                if focused_abs_key == *abs_key {
                    changed_key_set.insert((*abs_key).into()).unwrap()
                }
            }
        }

        changed_key_set.get_vector()
    }
}

pub(crate) struct DatabaseCore<
    'a,
    Focus,
    Mutex,
    Data,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> where
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
    Mutex: ScopedRawMutex + ConstInit,
    AbsKey: AbsKeyConstraints<AbsField>,
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
    pub(crate) subscribers:
        SubscriberData<'a, Mutex, AbsKey, FlatKey, FlatField, FLAT_PARAMETER_COUNT>,
    pub(crate) focus_handler: Focus,
    _fields: PhantomData<FlatField>,
}

impl<
    'a,
    Focus,
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
        Focus,
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
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
    Mutex: ScopedRawMutex + ConstInit,
    AbsKey: AbsKeyConstraints<AbsField>,
    AbsField: AbsFieldConstraints,
    FlatKey: FlatKeyConstraints<AbsKey, FlatField>,
    FlatField: FlatFieldConstraints<AbsField>,
    usize: UsizeConstraints<FlatKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
{
    const _STATIC_ASSERTIONS: () = {
        assert!(ABS_PARAMETER_COUNT == AbsKey::COUNT);
        assert!(ABS_PARAMETER_COUNT == AbsField::COUNT);
        assert!(FLAT_PARAMETER_COUNT == FlatKey::COUNT);
        assert!(FLAT_PARAMETER_COUNT == FlatField::COUNT);
    };

    pub(crate) const fn new(data: Data, focus_handler: Focus) -> Self {
        Self {
            focus_handler,
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

        let abs_key: AbsKey = abs_field.clone().into();

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
                .try_with(|other_internal| internal.clone(&other_internal, &self.focus_handler))
        }) {
            None => Err(DatabaseError::LockFail),
            Some(changes) => match changes {
                None => Err(DatabaseError::LockFail),
                Some(changes) => {
                    self.subscribers.on_changes(&changes);
                    Ok(())
                }
            },
        }
    }
}
