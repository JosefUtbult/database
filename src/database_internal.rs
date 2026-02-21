use core::{marker::PhantomData, panic};

use heapless::Vec;

use crate::{
    DataFieldAccessor,
    database_traits::{
        AbsFieldConstraints, AbsKeyConstraints, FlatFieldConstraints, FlatKeyConstraints,
        UsizeConstraints,
    },
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
    AbsKey: AbsKeyConstraints<AbsField, ABS_PARAMETER_COUNT>,
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
    AbsKey: AbsKeyConstraints<AbsField, ABS_PARAMETER_COUNT>,
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

    pub(crate) fn clone(&self, _other: &Self) {
        let changed_key_set: KeySet<FlatKey, FLAT_PARAMETER_COUNT> = KeySet::new();
        // for key in AbsKey::ALL_KEYS.iter() {
        //     let this_field = self.data.get(*key);
        //     let other_field = other.data.get(*key);

        //     if this_field != other_field && changes. {
        //     }
        // }
    }
}
