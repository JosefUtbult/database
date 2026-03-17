use core::marker::PhantomData;

use heapless::Vec;

use crate::{FlatKeyConstraints, database_core::DatabaseError};

pub trait DynamicKeySet<AbsKey, FlatKey>
where
    FlatKey: FlatKeyConstraints<AbsKey>,
{
    fn insert_flat_key(&mut self, key: FlatKey) -> Result<(), DatabaseError>;
    fn insert_abs_key(&mut self, key: AbsKey) -> Result<(), DatabaseError>;
}

pub struct KeySet<AbsKey, FlatKey, const KEY_COUNT: usize> {
    data: [Option<FlatKey>; KEY_COUNT],
    _abs_key: PhantomData<AbsKey>,
}

impl<AbsKey, FlatKey, const KEY_COUNT: usize> KeySet<AbsKey, FlatKey, KEY_COUNT>
where
    FlatKey: FlatKeyConstraints<AbsKey>,
{
    pub(crate) const fn new() -> Self {
        Self {
            data: [const { None }; KEY_COUNT],
            _abs_key: PhantomData,
        }
    }

    pub(crate) fn to_vector(self) -> Vec<FlatKey, KEY_COUNT> {
        let mut result = Vec::new();
        for mut element in self.data.into_iter() {
            if let Some(element) = element.take() {
                if result.push(element).is_err() {
                    panic!("Vector overflow");
                }
            }
        }
        result
    }
}

impl<AbsKey, FlatKey, const KEY_COUNT: usize> DynamicKeySet<AbsKey, FlatKey>
    for KeySet<AbsKey, FlatKey, KEY_COUNT>
where
    FlatKey: FlatKeyConstraints<AbsKey>,
{
    fn insert_flat_key(&mut self, key: FlatKey) -> Result<(), DatabaseError> {
        let index: usize = key.clone().to_usize();
        if index < KEY_COUNT {
            if self.data[index].is_none() {
                let _ = self.data[index].insert(key);
            }
            Ok(())
        } else {
            Err(DatabaseError::ParameterCountMissmatch)
        }
    }

    fn insert_abs_key(&mut self, key: AbsKey) -> Result<(), DatabaseError> {
        self.insert_flat_key(key.into())
    }
}
