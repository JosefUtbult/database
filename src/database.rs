use core::marker::PhantomData;
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::mutex::ScopedLocked;

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

pub struct DatabaseHandler<Mutex, Data, Key, Field>
where
    Mutex: ScopedRawMutex + ConstInit,
    Data: DataFieldAccessor<Key, Field>,
{
    data: ScopedLocked<Mutex, InternalMutable<Data, Key, Field>>,
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

impl<Mutex, Data, Key, Field> DatabaseHandler<Mutex, Data, Key, Field>
where
    Mutex: ScopedRawMutex + ConstInit,
    Data: DataFieldAccessor<Key, Field>,
{
    pub const fn new(data: Data) -> Self {
        Self {
            data: ScopedLocked::new(InternalMutable::new(data)),
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
        match self.data.try_with(|internal| internal.data.set(field)) {
            Some(()) => Ok(()),
            None => Err(DatabaseError::LockFail),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        DatabaseHandler,
        mutex::test_mutex::Mutex,
        test_types::test_types::{MyData, MyDataFields, MyDataKeys},
    };

    type MyDatabase = DatabaseHandler<Mutex, MyData, MyDataKeys, MyDataFields>;

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
