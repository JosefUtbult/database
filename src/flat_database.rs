use crate::{
    database_core::{AllKeys, DatabaseCore},
    database_traits::{
        AbsFieldConstraints, AbsKeyConstraints, FlatFieldConstraints, FlatKeyConstraints,
        UsizeConstraints,
    },
};

use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{DataFieldAccessor, database_internal::DatabaseError};

pub struct FlatDatabase<'a, Mutex, Data, Key, Field, const PARAMETER_COUNT: usize>(
    DatabaseCore<'a, Mutex, Data, Key, Field, Key, Field, PARAMETER_COUNT, PARAMETER_COUNT>,
)
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: AbsKeyConstraints<Field, PARAMETER_COUNT>,
    Field: AbsFieldConstraints,
    Key: FlatKeyConstraints<Key, Field>,
    Field: FlatFieldConstraints<Field>,
    usize: UsizeConstraints<Key>,
    Data: DataFieldAccessor<Key, Field>;

impl<'a, Mutex, Data, Key, Field, const PARAMETER_COUNT: usize>
    FlatDatabase<'a, Mutex, Data, Key, Field, PARAMETER_COUNT>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: AbsKeyConstraints<Field, PARAMETER_COUNT>,
    Field: AbsFieldConstraints,
    Key: FlatKeyConstraints<Key, Field>,
    Field: FlatFieldConstraints<Field>,
    usize: UsizeConstraints<Key>,
    Data: DataFieldAccessor<Key, Field>,
{
    pub const fn new(data: Data) -> Self {
        Self(DatabaseCore::new(data))
    }

    pub fn get(&self, key: Key) -> Result<Field, DatabaseError> {
        self.0.get(key)
    }

    pub fn set(&self, field: Field) -> Result<(), DatabaseError> {
        self.0.set(field.into(), field)
    }

    pub fn clone(&self, other: &Self) -> Result<(), DatabaseError> {
        self.0.clone(&other.0)
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
