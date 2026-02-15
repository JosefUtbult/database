use core::{hash::Hash, marker::PhantomData};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{
    DataFieldAccessor, FocusHandler, SubscriberData,
    database_internal::{DatabaseError, InternalMutable},
    mutex::ScopedLocked,
};

pub struct LayerDatabase<
    'a,
    Mutex,
    Data,
    Focus,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const FLAT_PARAMETER_COUNT: usize,
> where
    Mutex: ScopedRawMutex + ConstInit,
    FlatKey: Ord + Hash + Copy + From<AbsKey>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    data: ScopedLocked<Mutex, InternalMutable<Data, AbsKey, AbsField>>,
    focus_handler: Focus,
    subscribers: SubscriberData<'a, Mutex, FlatKey, FLAT_PARAMETER_COUNT>,
    _keys: PhantomData<FlatKey>,
    _fields: PhantomData<FlatField>,
}

impl<
    'a,
    Mutex,
    Data,
    Focus,
    AbsKey,
    AbsField,
    FlatKey,
    FlatField,
    const FLAT_PARAMETER_COUNT: usize,
>
    LayerDatabase<
        'a,
        Mutex,
        Data,
        Focus,
        AbsKey,
        AbsField,
        FlatKey,
        FlatField,
        FLAT_PARAMETER_COUNT,
    >
where
    Mutex: ScopedRawMutex + ConstInit,
    AbsKey: From<AbsField> + Copy,
    AbsField: Eq + PartialEq + Copy,
    FlatKey: Ord + Hash + Copy + From<FlatField> + From<AbsKey>,
    FlatField: Copy + Eq + PartialEq + From<AbsField>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    pub const fn new(data: Data, focus_handler: Focus) -> Self {
        Self {
            data: ScopedLocked::new(InternalMutable::new(data)),
            focus_handler,
            subscribers: SubscriberData::new(),
            _keys: PhantomData,
            _fields: PhantomData,
        }
    }

    pub fn get_focus_handler(&self) -> &Focus {
        &self.focus_handler
    }

    pub fn get_absolute(&self, key: AbsKey) -> Result<FlatField, DatabaseError> {
        match self.data.try_with(|internal| {
            let field = internal.data.get(key);
            field.into()
        }) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub fn get(&self, key: FlatKey) -> Result<FlatField, DatabaseError> {
        let abs_key = self.focus_handler.get_focus_key(key);
        self.get_absolute(abs_key)
    }

    pub fn set_absolute(&self, field: AbsField) -> Result<(), DatabaseError> {
        #[derive(PartialEq, Eq)]
        enum SetState {
            Updated,
            UpToDate,
        }

        let flat_field: FlatField = field.into();
        let flat_key: FlatKey = flat_field.into();
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

    pub fn set(&self, field: FlatField) -> Result<(), DatabaseError> {
        let abs_field = self.focus_handler.get_focus_field(field);
        self.set_absolute(abs_field)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        LayerDatabase,
        mutex::test_mutex::Mutex,
        test_data_field_accessor::{InnerFocus, MyFocusHandler},
        test_types::test_types::{
            MY_DATA_PARAMETER_FLAT_COUNT, MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields,
            MyDataFlatKeys, MyInnerDataFields, MyInnerDataKeys, MyLayerData,
        },
    };

    type MyDatabase<'a> = LayerDatabase<
        'a,
        Mutex,
        MyLayerData,
        MyFocusHandler,
        MyDataAbsKeys,
        MyDataAbsFields,
        MyDataFlatKeys,
        MyDataFlatFields,
        MY_DATA_PARAMETER_FLAT_COUNT,
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
                .data
                .with(|internal| { internal.data.inner1.param4 }),
            res1
        );

        std::println!(
            "Database inner 2 param 4: {}. Flat res {:?}",
            database
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
