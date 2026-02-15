use core::{hash::Hash, marker::PhantomData};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{FocusHandler, SubscriberData, mutex::ScopedLocked};

pub trait DataFieldAccessor<AbsKey, AbsField> {
    fn get(&self, key: AbsKey) -> AbsField;
    fn set(&mut self, field: AbsField);
}

struct InternalMutable<Data, Focus, AbsKey, AbsField, FlatKey, FlatField>
where
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    data: Data,
    focus_handler: Focus,
    _abs_key: PhantomData<AbsKey>,
    _abs_fields: PhantomData<AbsField>,
    _flat_key: PhantomData<FlatKey>,
    _flat_fields: PhantomData<FlatField>,
}

pub struct DatabaseHandler<
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
    AbsField: Eq + PartialEq,
    FlatKey: Ord + Hash + Copy + From<AbsKey>,
    FlatField: Copy + Eq + PartialEq + From<AbsField>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    data: ScopedLocked<Mutex, InternalMutable<Data, Focus, AbsKey, AbsField, FlatKey, FlatField>>,
    subscribers: SubscriberData<'a, Mutex, FlatKey, FLAT_PARAMETER_COUNT>,
    _keys: PhantomData<FlatKey>,
    _fields: PhantomData<FlatField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseError {
    LockFail,
    SubscriberOverflow,
}

impl<Data, Focus, AbsKey, AbsField, FlatKey, FlatField>
    InternalMutable<Data, Focus, AbsKey, AbsField, FlatKey, FlatField>
where
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    pub const fn new(data: Data, focus_handler: Focus) -> Self {
        Self {
            data,
            focus_handler,
            _abs_key: PhantomData,
            _abs_fields: PhantomData,
            _flat_key: PhantomData,
            _flat_fields: PhantomData,
        }
    }
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
    DatabaseHandler<
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
    AbsKey: From<AbsField>,
    AbsField: Eq + PartialEq,
    FlatKey: Ord + Hash + Copy + From<FlatField> + From<AbsKey>,
    FlatField: Copy + Eq + PartialEq + From<AbsField>,
    Data: DataFieldAccessor<AbsKey, AbsField>,
    Focus: FocusHandler<AbsKey, AbsField, FlatKey, FlatField>,
{
    pub const fn new(data: Data, focus_handler: Focus) -> Self {
        Self {
            data: ScopedLocked::new(InternalMutable::new(data, focus_handler)),
            subscribers: SubscriberData::new(),
            _keys: PhantomData,
            _fields: PhantomData,
        }
    }

    pub fn get(&self, key: FlatKey) -> Result<FlatField, DatabaseError> {
        match self.data.try_with(|internal| {
            let abs_key = internal.focus_handler.get_focus_key(key);
            let field = internal.data.get(abs_key);
            field.into()
        }) {
            Some(field) => Ok(field),
            None => Err(DatabaseError::LockFail),
        }
    }

    pub fn set(&self, field: FlatField) -> Result<(), DatabaseError> {
        #[derive(PartialEq, Eq)]
        enum SetState {
            Updated,
            UpToDate,
        }

        match self.data.try_with(|internal| {
            let flat_key: FlatKey = field.into();
            let abs_key = internal.focus_handler.get_focus_key(flat_key);
            let abs_field = internal.focus_handler.get_focus_field(field);
            if internal.data.get(abs_key) != abs_field {
                internal.data.set(abs_field);
                (SetState::Updated, flat_key)
            } else {
                (SetState::UpToDate, flat_key)
            }
        }) {
            None => Err(DatabaseError::LockFail),
            Some((set_state, flat_key)) => {
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
        DatabaseHandler, FocusHandler,
        mutex::test_mutex::Mutex,
        test_types::test_types::{
            MY_DATA_PARAMETER_FLAT_COUNT, MyData, MyDataAbsFields, MyDataAbsKeys, MyDataFlatFields,
            MyDataFlatKeys, MyInnerDataFields, MyInnerDataKeys,
        },
    };

    struct MyFocusHandler {}

    impl MyFocusHandler {
        const fn new() -> Self {
            Self {}
        }
    }

    impl FocusHandler<MyDataAbsKeys, MyDataAbsFields, MyDataFlatKeys, MyDataFlatFields>
        for MyFocusHandler
    {
        fn get_focus_key(&self, key: MyDataFlatKeys) -> MyDataAbsKeys {
            match key {
                MyDataFlatKeys::Param1 => MyDataAbsKeys::Param1,
                MyDataFlatKeys::Param2 => MyDataAbsKeys::Param2,
                MyDataFlatKeys::Param3 => MyDataAbsKeys::Param3,
                MyDataFlatKeys::Param4 => MyDataAbsKeys::Inner1(MyInnerDataKeys::Param4),
                MyDataFlatKeys::Param5 => MyDataAbsKeys::Inner1(MyInnerDataKeys::Param5),
            }
        }

        fn get_focus_field(&self, field: MyDataFlatFields) -> MyDataAbsFields {
            match field {
                MyDataFlatFields::Param1(value) => MyDataAbsFields::Param1(value),
                MyDataFlatFields::Param2(value) => MyDataAbsFields::Param2(value),
                MyDataFlatFields::Param3(value) => MyDataAbsFields::Param3(value),
                MyDataFlatFields::Param4(value) => {
                    MyDataAbsFields::Inner1(MyInnerDataFields::Param4(value))
                }
                MyDataFlatFields::Param5(value) => {
                    MyDataAbsFields::Inner1(MyInnerDataFields::Param5(value))
                }
            }
        }
    }

    type MyDatabase<'a> = DatabaseHandler<
        'a,
        Mutex,
        MyData,
        MyFocusHandler,
        MyDataAbsKeys,
        MyDataAbsFields,
        MyDataFlatKeys,
        MyDataFlatFields,
        MY_DATA_PARAMETER_FLAT_COUNT,
    >;

    fn build_database<'a>() -> MyDatabase<'a> {
        MyDatabase::new(MyData::new(), MyFocusHandler::new())
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
