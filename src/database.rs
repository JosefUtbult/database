use core::{hash::Hash, marker::PhantomData};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{SubscriberData, mutex::ScopedLocked};

pub trait DataFieldAccessor<Key, Field> {
    fn get(&self, key: Key) -> Field;
    fn set(&mut self, field: Field);
}

struct InternalMutable<Data, Key, Field>
where
    Data: DataFieldAccessor<Key, Field>,
{
    data: Data,
    _keys: PhantomData<Key>,
    _fields: PhantomData<Field>,
}

pub struct DatabaseHandler<'a, Mutex, Data, Key, Field, const PARAMETER_COUNT: usize>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: Ord + Hash + Copy,
    Field: Copy + Eq + PartialEq,
    Data: DataFieldAccessor<Key, Field>,
{
    data: ScopedLocked<Mutex, InternalMutable<Data, Key, Field>>,
    subscribers: SubscriberData<'a, Mutex, Key, PARAMETER_COUNT>,
    _keys: PhantomData<Key>,
    _fields: PhantomData<Field>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    LockFail,
    SubscriberOverflow,
}

impl<Data, Key, Field> InternalMutable<Data, Key, Field>
where
    Data: DataFieldAccessor<Key, Field>,
{
    pub const fn new(data: Data) -> Self {
        Self {
            data,
            _keys: PhantomData,
            _fields: PhantomData,
        }
    }
}

impl<'a, Mutex, Data, Key, Field, const PARAMETER_COUNT: usize>
    DatabaseHandler<'a, Mutex, Data, Key, Field, PARAMETER_COUNT>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: Ord + Hash + Copy + From<Field>,
    Field: Copy + Eq + PartialEq,
    Data: DataFieldAccessor<Key, Field>,
{
    pub const fn new(data: Data) -> Self {
        Self {
            data: ScopedLocked::new(InternalMutable::new(data)),
            subscribers: SubscriberData::new(),
            _keys: PhantomData,
            _fields: PhantomData,
        }
    }

    pub fn get(&self, key: Key) -> Result<Field, DatabaseError> {
        match self.data.try_with(|internal| internal.data.get(key)) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub fn set(&self, field: Field) -> Result<(), DatabaseError> {
        #[derive(PartialEq, Eq)]
        enum SetState {
            Updated,
            UpToDate,
        }

        let key: Key = field.into();
        match self.data.try_with(|internal| {
            if internal.data.get(key) != field {
                internal.data.set(field);
                SetState::Updated
            } else {
                SetState::UpToDate
            }
        }) {
            None => Err(DatabaseError::LockFail),
            Some(set_state) => {
                if set_state == SetState::Updated {
                    self.subscribers.on_change(key);
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        DatabaseHandler,
        mutex::test_mutex::Mutex,
        test_types::test_types::{MY_DATA_PARAMETER_COUNT, MyData, MyDataFields, MyDataKeys},
    };

    type MyDatabase<'a> =
        DatabaseHandler<'a, Mutex, MyData, MyDataKeys, MyDataFields, MY_DATA_PARAMETER_COUNT>;

    #[test]
    fn create_database() {
        let _database = MyDatabase::new(MyData::new());
    }

    #[test]
    fn set_data() {
        let database = MyDatabase::new(MyData::new());
        database.set(MyDataFields::Param1(1)).unwrap();
    }

    #[test]
    fn get_data() {
        let database = MyDatabase::new(MyData::new());
        let _ = database.get(MyDataKeys::Param1).unwrap();
    }

    #[test]
    fn set_get_data() {
        let database = MyDatabase::new(MyData::new());
        database.set(MyDataFields::Param1(1)).unwrap();
        let res = database.get(MyDataKeys::Param1).unwrap();
        assert!(matches!(res, MyDataFields::Param1(1)));
    }
}
