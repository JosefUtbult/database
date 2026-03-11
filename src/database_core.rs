use heapless::Vec;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    AllVariants, DataFieldAccessor, DatabaseDescription, FocusHandler, SubscriberData, ToFromUsize,
    ToKey, VariantCount, mutex::ScopedLocked,
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
    Key: Eq + Copy + ToFromUsize,
{
    const fn new() -> Self {
        Self([const { None }; KEY_COUNT])
    }

    fn insert(&mut self, key: Key) -> Result<(), DatabaseError> {
        let index: usize = key.clone().to_usize();
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
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> {
    pub(crate) data: Database::Data,
}

impl<
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> InternalMutable<Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>
{
    pub(crate) const fn new(data: Database::Data) -> Self {
        Self { data }
    }

    pub(crate) fn clone<Focus>(
        &mut self,
        other: &Self,
        focus_handler: &Focus,
    ) -> Vec<Database::FlatKey, FLAT_PARAMETER_COUNT>
    where
        Focus: FocusHandler<Database>,
    {
        let mut changed_key_set: KeySet<Database::FlatKey, FLAT_PARAMETER_COUNT> = KeySet::new();
        for abs_key in <Database::AbsKey as AllVariants>::ALL_VARIANTS.iter() {
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
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> {
    pub(crate) data:
        ScopedLocked<Mutex, InternalMutable<Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>>,
    pub(crate) subscribers: SubscriberData<'a, Mutex, Database, FLAT_PARAMETER_COUNT>,
    pub(crate) focus_handler: Focus,
}

impl<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> DatabaseCore<'a, Focus, Mutex, Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>
{
    const _STATIC_ASSERTIONS: () = {
        assert!(ABS_PARAMETER_COUNT == <Database::AbsKey as VariantCount>::COUNT);
        assert!(ABS_PARAMETER_COUNT == <Database::AbsField as VariantCount>::COUNT);
        assert!(FLAT_PARAMETER_COUNT == <Database::FlatKey as VariantCount>::COUNT);
        assert!(FLAT_PARAMETER_COUNT == <Database::FlatField as VariantCount>::COUNT);
    };

    pub(crate) const fn new(data: Database::Data, focus_handler: Focus) -> Self {
        Self {
            focus_handler,
            data: ScopedLocked::new(InternalMutable::new(data)),
            subscribers: SubscriberData::new(),
        }
    }

    pub(crate) fn get(&self, key: Database::AbsKey) -> Result<Database::FlatField, DatabaseError> {
        match self.data.try_with(|internal| {
            let field = internal.data.get(key);
            field.into()
        }) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub(crate) fn set(
        &self,
        flat_key: Database::FlatKey,
        abs_field: Database::AbsField,
    ) -> Result<(), DatabaseError> {
        #[derive(PartialEq, Eq)]
        enum SetState {
            Updated,
            UpToDate,
        }

        let abs_key: Database::AbsKey = abs_field.to_key();

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
