use crate::{
    DatabaseDescription, FlatDatabaseDescription, FocusHandler, Subscriber, SubscriberError,
    database_core::{DatabaseCore, DatabaseError},
};

use mutex_traits::{ConstInit, ScopedRawMutex};

struct FlatFocusHandler {}
impl<Database: FlatDatabaseDescription> FocusHandler<Database> for FlatFocusHandler {
    fn get_focus_key(
        &self,
        key: <Database as DatabaseDescription>::FlatKey,
    ) -> <Database as DatabaseDescription>::AbsKey {
        key
    }

    fn get_focus_field(
        &self,
        field: <Database as DatabaseDescription>::FlatField,
    ) -> <Database as DatabaseDescription>::AbsField {
        field
    }
}

pub struct FlatDatabase<
    'a,
    Mutex: ScopedRawMutex + ConstInit,
    Database: FlatDatabaseDescription,
    const PARAMETER_COUNT: usize,
>(DatabaseCore<'a, FlatFocusHandler, Mutex, Database, PARAMETER_COUNT, PARAMETER_COUNT>);

impl<
    'a,
    Mutex: ScopedRawMutex + ConstInit,
    Database: FlatDatabaseDescription,
    const PARAMETER_COUNT: usize,
> FlatDatabase<'a, Mutex, Database, PARAMETER_COUNT>
{
    pub const fn new(data: Database::Data) -> Self {
        Self(DatabaseCore::new(data, FlatFocusHandler {}))
    }

    pub fn subscribe(
        &self,
        subscriber: &'a dyn Subscriber<Database::Key>,
        key: Database::Key,
    ) -> Result<(), SubscriberError> {
        self.0.subscribe(subscriber, key)
    }

    pub fn notify_subscribers(&self) {
        self.0.notify_subscribers();
    }

    pub fn get(&self, key: Database::Key) -> Result<Database::Field, DatabaseError> {
        self.0.get_absolute(key)
    }

    pub fn set(&self, field: Database::Field) -> Result<(), DatabaseError> {
        self.0.set_absolute(field)
    }

    pub fn clone(&self, other: &Database::Data) -> Result<(), DatabaseError> {
        self.0.clone(&other)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        FlatDatabase,
        mutex::test_mutex::Mutex,
        test_types::{
            TEST_FLAT_DATABASE_COUNT, TestFlatDatabaseDescription, create_test_flat_data,
            flat::{MyFlatFields, MyFlatKeys},
        },
    };

    type MyDatabase<'a> =
        FlatDatabase<'a, Mutex, TestFlatDatabaseDescription, TEST_FLAT_DATABASE_COUNT>;

    fn build_database<'a>() -> MyDatabase<'a> {
        MyDatabase::new(create_test_flat_data())
    }

    #[test]
    fn create_database() {
        let _database = build_database();
    }

    #[test]
    fn set_data() {
        let database = build_database();
        database.set(MyFlatFields::Param1(1)).unwrap();
    }

    #[test]
    fn get_data() {
        let database = build_database();
        let _ = database.get(MyFlatKeys::Param1).unwrap();
    }

    #[test]
    fn set_get_data() {
        let database = build_database();
        database.set(MyFlatFields::Param1(1)).unwrap();
        let res = database.get(MyFlatKeys::Param1).unwrap();
        assert!(matches!(res, MyFlatFields::Param1(1)));
    }
}
