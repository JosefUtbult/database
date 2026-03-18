use crate::{
    DatabaseDescription, FocusHandler,
    database_core::{DatabaseCore, DatabaseError},
};

use mutex_traits::{ConstInit, ScopedRawMutex};

pub struct LayerDatabase<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> {
    database_core:
        DatabaseCore<'a, Focus, Mutex, Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>,
}

impl<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> LayerDatabase<'a, Focus, Mutex, Database, ABS_PARAMETER_COUNT, FLAT_PARAMETER_COUNT>
{
    pub const fn new(data: Database::Data, focus_handler: Focus) -> Self {
        Self {
            database_core: DatabaseCore::new(data, focus_handler),
        }
    }

    pub fn get_focus_handler(&self) -> &Focus {
        &self.database_core.focus_handler
    }

    pub fn get_absolute(
        &self,
        key: Database::AbsKey,
    ) -> Result<Database::FlatField, DatabaseError> {
        self.database_core.get_absolute(key)
    }

    pub fn get(&self, key: Database::FlatKey) -> Result<Database::FlatField, DatabaseError> {
        let abs_key = self.database_core.focus_handler.get_focus_key(key);
        self.get_absolute(abs_key)
    }

    pub fn set_absolute(&self, field: Database::AbsField) -> Result<(), DatabaseError> {
        self.database_core.set_absolute(field)
    }

    pub fn set(&self, field: Database::FlatField) -> Result<(), DatabaseError> {
        let abs_field = self.database_core.focus_handler.get_focus_field(field);
        self.set_absolute(abs_field)
    }

    pub fn clone(&self, other: &Self) -> Result<(), DatabaseError> {
        self.database_core.clone(&other.database_core)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::{
        LayerDatabase,
        focus_handler::FolderFocus,
        layer::{AbsFields, AbsKeys, Fields, Keys, MyInnerDataFocus, MyLayerData, my_inner_data},
        mutex::test_mutex::Mutex,
        test_data_field_accessor::MyFocusHandler,
        test_types::{
            TEST_LAYER_DATABASE_ABS_COUNT, TEST_LAYER_DATABASE_FLAT_COUNT,
            TestLayerDatabaseDescription,
        },
    };

    type MyDatabase<'a> = LayerDatabase<
        'a,
        MyFocusHandler,
        Mutex,
        TestLayerDatabaseDescription,
        TEST_LAYER_DATABASE_ABS_COUNT,
        TEST_LAYER_DATABASE_FLAT_COUNT,
    >;

    fn build_database<'a>() -> MyDatabase<'a> {
        MyDatabase::new(MyLayerData::new(), MyFocusHandler::new())
    }

    #[test]
    fn create_database() {
        let _database = build_database();
    }

    #[test]
    fn set_data() {
        let database = build_database();
        database.set(Fields::Param1(1)).unwrap();
    }

    #[test]
    fn set_absolute_data() {
        let database = build_database();
        database.set_absolute(AbsFields::Param1(1)).unwrap();
    }

    #[test]
    fn get_data() {
        let database = build_database();
        let _ = database.get(Keys::Param1).unwrap();
    }

    #[test]
    fn get_absolute_data() {
        let database = build_database();
        let _ = database.get_absolute(AbsKeys::Param1).unwrap();
    }

    #[test]
    fn set_get_data() {
        let database = build_database();
        database.set(Fields::Param1(1)).unwrap();
        let res = database.get(Keys::Param1).unwrap();
        assert!(matches!(res, Fields::Param1(1)));
    }

    #[test]
    fn set_get_absolute_data() {
        let database = build_database();
        database
            .set_absolute(AbsFields::Inner1(my_inner_data::Fields::Param4(1)))
            .unwrap();
        database
            .set_absolute(AbsFields::Inner2(my_inner_data::Fields::Param4(2)))
            .unwrap();

        let res1 = database
            .get_absolute(AbsKeys::Inner1(my_inner_data::Keys::Param4))
            .unwrap();
        let res2 = database
            .get_absolute(AbsKeys::Inner2(my_inner_data::Keys::Param4))
            .unwrap();

        std::println!(
            "Database inner 1 param 4: {}. Flat res {:?}",
            database
                .database_core
                .data
                .with(|internal| { internal.data.inner1.param4 }),
            res1
        );

        std::println!(
            "Database inner 2 param 4: {}. Flat res {:?}",
            database
                .database_core
                .data
                .with(|internal| { internal.data.inner2.param4 }),
            res2
        );

        assert!(matches!(res1, Fields::Param4(1)));
        assert!(matches!(res2, Fields::Param4(2)));
    }

    #[test]
    fn get_data_focus_change() {
        let database = build_database();
        database
            .set_absolute(AbsFields::Inner1(my_inner_data::Fields::Param4(1)))
            .unwrap();

        database
            .set_absolute(AbsFields::Inner2(my_inner_data::Fields::Param4(2)))
            .unwrap();

        let res1 = database.get(Keys::Param4).unwrap();
        database.set(Fields::Param4(2)).unwrap();
        let res2 = database.get(Keys::Param4).unwrap();

        assert!(matches!(res1, Fields::Param4(1)));
        assert!(matches!(res2, Fields::Param4(2)));
    }

    #[test]
    fn set_data_focus_change() {
        let database = build_database();
        database.set(Fields::Param4(1)).unwrap();

        database
            .get_focus_handler()
            .set_focus(MyInnerDataFocus::Inner2);

        database.set(Fields::Param4(2)).unwrap();

        let res1 = database
            .get_absolute(AbsKeys::Inner1(my_inner_data::Keys::Param4))
            .unwrap();

        let res2 = database
            .get_absolute(AbsKeys::Inner2(my_inner_data::Keys::Param4))
            .unwrap();

        assert!(matches!(res1, Fields::Param4(1)));
        assert!(matches!(res2, Fields::Param4(2)));
    }

    #[test]
    fn set_get_data_focus_change() {
        let database = build_database();
        let focus_handler = database.get_focus_handler();

        database.set(Fields::Param4(1)).unwrap();

        focus_handler.set_focus(MyInnerDataFocus::Inner2);
        database.set(Fields::Param4(2)).unwrap();

        focus_handler.set_focus(MyInnerDataFocus::Inner1);
        let res1 = database.get(Keys::Param4).unwrap();

        focus_handler.set_focus(MyInnerDataFocus::Inner2);
        let res2 = database.get(Keys::Param4).unwrap();

        assert!(matches!(res1, Fields::Param4(1)));
        assert!(matches!(res2, Fields::Param4(2)));
    }
}
