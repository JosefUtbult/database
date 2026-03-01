use crate::{
    DatabaseDescription, FocusHandler,
    database_core::{DatabaseCore, DatabaseError},
    database_traits::{ToKey, UsizeConstraints},
};

use mutex_traits::{ConstInit, ScopedRawMutex};

pub struct LayerDatabase<
    'a,
    Focus: FocusHandler<Database>,
    Mutex: ScopedRawMutex + ConstInit,
    Database: DatabaseDescription,
    const ABS_PARAMETER_COUNT: usize,
    const FLAT_PARAMETER_COUNT: usize,
> where
    usize: UsizeConstraints<Database::FlatKey>,
{
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
where
    usize: UsizeConstraints<Database::FlatKey>,
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
        self.database_core.get(key)
    }

    pub fn get(&self, key: Database::FlatKey) -> Result<Database::FlatField, DatabaseError> {
        let abs_key = self.database_core.focus_handler.get_focus_key(key);
        self.get_absolute(abs_key)
    }

    pub fn set_absolute(&self, field: Database::AbsField) -> Result<(), DatabaseError> {
        let flat_field: Database::FlatField = field.clone().into();
        let flat_key: Database::FlatKey = flat_field.to_key();
        self.database_core.set(flat_key, field)
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
        DatabaseDescription, LayerDatabase,
        mutex::test_mutex::Mutex,
        test_data_field_accessor::{InnerFocus, MyFocusHandler},
        test_types::test_types::{
            MY_DATA_ABS_VARIANT_COUNT, MY_DATA_FLAT_VARIANT_COUNT, MyDataAbsFields,
            MyDataAbsFolders, MyDataAbsKeys, MyDataFlatFields, MyDataFlatFolders, MyDataFlatKeys,
            MyInnerDataFields, MyInnerDataKeys, MyLayerData,
        },
    };

    pub(crate) struct MyDatabaseDescription {}
    impl DatabaseDescription for MyDatabaseDescription {
        type AbsKey = MyDataAbsKeys;
        type AbsField = MyDataAbsFields;
        type AbsFolder = MyDataAbsFolders;
        type FlatKey = MyDataFlatKeys;
        type FlatField = MyDataFlatFields;
        type FlatFolder = MyDataFlatFolders;
        type Data = MyLayerData;
    }

    type MyDatabase<'a> = LayerDatabase<
        'a,
        MyFocusHandler,
        Mutex,
        MyDatabaseDescription,
        MY_DATA_ABS_VARIANT_COUNT,
        MY_DATA_FLAT_VARIANT_COUNT,
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
        database.set(MyDataFlatFields::Param1(1)).unwrap();
    }

    #[test]
    fn set_absolute_data() {
        let database = build_database();
        database.set_absolute(MyDataAbsFields::Param1(1)).unwrap();
    }

    #[test]
    fn get_data() {
        let database = build_database();
        let _ = database.get(MyDataFlatKeys::Param1).unwrap();
    }

    #[test]
    fn get_absolute_data() {
        let database = build_database();
        let _ = database.get_absolute(MyDataAbsKeys::Param1).unwrap();
    }

    #[test]
    fn set_get_data() {
        let database = build_database();
        database.set(MyDataFlatFields::Param1(1)).unwrap();
        let res = database.get(MyDataFlatKeys::Param1).unwrap();
        assert!(matches!(res, MyDataFlatFields::Param1(1)));
    }

    #[test]
    fn set_get_absolute_data() {
        let database = build_database();
        database
            .set_absolute(MyDataAbsFields::Inner1(MyInnerDataFields::Param4(1)))
            .unwrap();
        database
            .set_absolute(MyDataAbsFields::Inner2(MyInnerDataFields::Param4(2)))
            .unwrap();

        let res1 = database
            .get_absolute(MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4))
            .unwrap();
        let res2 = database
            .get_absolute(MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4))
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

        assert!(matches!(res1, MyDataFlatFields::Param4(1)));
        assert!(matches!(res2, MyDataFlatFields::Param4(2)));
    }

    #[test]
    fn get_data_focus_change() {
        let database = build_database();
        database
            .set_absolute(MyDataAbsFields::Inner1(MyInnerDataFields::Param4(1)))
            .unwrap();

        database
            .set_absolute(MyDataAbsFields::Inner2(MyInnerDataFields::Param4(2)))
            .unwrap();

        let res1 = database.get(MyDataFlatKeys::Param4).unwrap();
        database.set(MyDataFlatFields::Param4(2)).unwrap();
        let res2 = database.get(MyDataFlatKeys::Param4).unwrap();

        assert!(matches!(res1, MyDataFlatFields::Param4(1)));
        assert!(matches!(res2, MyDataFlatFields::Param4(2)));
    }

    #[test]
    fn set_data_focus_change() {
        let database = build_database();
        database.set(MyDataFlatFields::Param4(1)).unwrap();

        database
            .get_focus_handler()
            .set_inner_focus(InnerFocus::Two);

        database.set(MyDataFlatFields::Param4(2)).unwrap();

        let res1 = database
            .get_absolute(MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4))
            .unwrap();
        let res2 = database
            .get_absolute(MyDataAbsKeys::Inner2(MyInnerDataKeys::Param4))
            .unwrap();

        assert!(matches!(res1, MyDataFlatFields::Param4(1)));
        assert!(matches!(res2, MyDataFlatFields::Param4(2)));
    }

    #[test]
    fn set_get_data_focus_change() {
        let database = build_database();
        let focus_handler = database.get_focus_handler();

        database.set(MyDataFlatFields::Param4(1)).unwrap();

        focus_handler.set_inner_focus(InnerFocus::Two);
        database.set(MyDataFlatFields::Param4(2)).unwrap();

        focus_handler.set_inner_focus(InnerFocus::One);
        let res1 = database.get(MyDataFlatKeys::Param4).unwrap();

        focus_handler.set_inner_focus(InnerFocus::Two);
        let res2 = database.get(MyDataFlatKeys::Param4).unwrap();

        assert!(matches!(res1, MyDataFlatFields::Param4(1)));
        assert!(matches!(res2, MyDataFlatFields::Param4(2)));
    }
}
