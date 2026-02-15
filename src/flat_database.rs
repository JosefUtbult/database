use core::{hash::Hash, marker::PhantomData};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, SubscriberData,
    database_internal::{DatabaseError, InternalMutable},
    mutex::ScopedLocked,
};

pub struct FlatDatabase<'a, Mutex, Data, Key, Field, const PARAMETER_COUNT: usize>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: Ord + Hash + Copy,
    Data: DataFieldAccessor<Key, Field>,
{
    data: ScopedLocked<Mutex, InternalMutable<Data, Key, Field>>,
    subscribers: SubscriberData<'a, Mutex, Key, PARAMETER_COUNT>,
    _keys: PhantomData<Key>,
    _fields: PhantomData<Field>,
}

impl<'a, Mutex, Data, Key, Field, const FLAT_PARAMETER_COUNT: usize>
    FlatDatabase<'a, Mutex, Data, Key, Field, FLAT_PARAMETER_COUNT>
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
        match self.data.try_with(|internal| {
            let field = internal.data.get(key);
            field.into()
        }) {
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

        let flat_field: Field = field.into();
        let flat_key: Key = flat_field.into();
        let abs_key = field.into();

        match self.data.try_with(|internal| {
            if internal.data.get(abs_key) != field {
                internal.data.set(field);
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

#[cfg(test)]
mod test {
    use crate::{
        FlatDatabase,
        mutex::test_mutex::Mutex,
        test_types::test_types::{
            MY_DATA_PARAMETER_FLAT_COUNT, MyDataFlatFields, MyDataFlatKeys, MyFlatData,
        },
    };

    type MyDatabase<'a> = FlatDatabase<
        'a,
        Mutex,
        MyFlatData,
        MyDataFlatKeys,
        MyDataFlatFields,
        MY_DATA_PARAMETER_FLAT_COUNT,
    >;

    fn build_database<'a>() -> MyDatabase<'a> {
        MyDatabase::new(MyFlatData::new())
    }

    #[test]
    fn create_database() {
        let _database = build_database();
    }

    #[test]
    fn set_data() {
        let database = build_database();
        database.set(MyDataFlatFields::Param1(1)).unwrap();
    }

    #[test]
    fn get_data() {
        let database = build_database();
        let _ = database.get(MyDataFlatKeys::Param1).unwrap();
    }

    #[test]
    fn set_get_data() {
        let database = build_database();
        database.set(MyDataFlatFields::Param1(1)).unwrap();
        let res = database.get(MyDataFlatKeys::Param1).unwrap();
        assert!(matches!(res, MyDataFlatFields::Param1(1)));
    }
}
