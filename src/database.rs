use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use critical_section::{Mutex as CriticalMutex, with as critical};
use spin::Mutex as SpinMutex;

use crate::{
    content::DatabaseContent, database_error::DatabaseError,
    subscriber_handler::DatabaseSubscriberHandler,
};

pub type ParameterChangeList<Parameter, const PARAMETER_COUNT: usize> =
    [Option<Parameter>; PARAMETER_COUNT];

pub trait DatabaseRef<Parameter>
where
    Parameter: Clone + Copy + Eq,
{
    fn internal_get(&self, parameter: &Parameter) -> Parameter;
}

pub struct Database<
    InternalContent,
    InternalSubscriberHandler,
    Parameter,
    const PARAMETER_COUNT: usize,
> where
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
    InternalSubscriberHandler:
        DatabaseSubscriberHandler<InternalContent, Parameter, PARAMETER_COUNT>,
    Parameter: Clone + Copy + Eq,
{
    content: CriticalMutex<RefCell<InternalContent>>,
    change_list: CriticalMutex<RefCell<ParameterChangeList<Parameter, PARAMETER_COUNT>>>,
    subscriber_handler: SpinMutex<RefCell<InternalSubscriberHandler>>,
    has_changed: AtomicBool,
}

impl<InternalContent, InternalSubscriberHandler, Parameter, const PARAMETER_COUNT: usize>
    DatabaseRef<Parameter>
    for Database<InternalContent, InternalSubscriberHandler, Parameter, PARAMETER_COUNT>
where
    Parameter: Copy + Clone + Eq,
    usize: From<Parameter>,
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
    InternalSubscriberHandler:
        DatabaseSubscriberHandler<InternalContent, Parameter, PARAMETER_COUNT>,
{
    fn internal_get(&self, parameter: &Parameter) -> Parameter {
        self.get(parameter)
    }
}

impl<InternalContent, InternalSubscriberHandler, Parameter, const PARAMETER_COUNT: usize>
    Database<InternalContent, InternalSubscriberHandler, Parameter, PARAMETER_COUNT>
where
    Parameter: Copy + Clone + Eq,
    usize: From<Parameter>,
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
    InternalSubscriberHandler:
        DatabaseSubscriberHandler<InternalContent, Parameter, PARAMETER_COUNT>,
{
    pub const fn new(
        content: InternalContent,
        subscriber_handler: InternalSubscriberHandler,
    ) -> Self {
        Self {
            content: CriticalMutex::new(RefCell::new(content)),
            change_list: CriticalMutex::new(RefCell::new([const { None }; PARAMETER_COUNT])),
            subscriber_handler: SpinMutex::new(RefCell::new(subscriber_handler)),
            has_changed: AtomicBool::new(false),
        }
    }

    pub fn get(&self, parameter: &Parameter) -> Parameter {
        critical(|cs| {
            let internal = self.content.borrow(cs).borrow();
            internal.get(parameter)
        })
    }

    pub fn set(&self, parameters: &[Parameter]) {
        let mut has_changed = false;

        critical(|cs| {
            let mut internal = self.content.borrow(cs).borrow_mut();
            let mut change_list = self.change_list.borrow(cs).borrow_mut();
            for parameter in parameters {
                // Swap out the value in the internal database content
                let reference = parameter.clone();
                let current_value = internal.get(&reference);
                internal.set(reference);

                // Check if the state has changed
                if reference != current_value {
                    has_changed = true;
                }

                let cloned_parameter = parameter.clone();
                let index: usize = cloned_parameter.into();

                // This should hard fail, as the default proc-macro implementation won't allow this
                assert!(index < PARAMETER_COUNT);
                let _ = change_list[index].insert(cloned_parameter);
            }
        });

        if has_changed {
            self.has_changed.store(true, Ordering::SeqCst);
        }
    }

    pub fn notify_subscribers(&self) -> Result<(), DatabaseError> {
        // Get the has set flag and clear it in one operation to see if something has changed
        if self.has_changed.swap(false, Ordering::SeqCst) {
            // Retrieve a copy of the change list. This is done in a critical section
            let parameter_change = critical(|cs| {
                // Clone the resulting parameter change list
                let mut parameter_change = self.change_list.borrow(cs).borrow_mut();
                let clone = parameter_change.clone();

                // Clear the original
                for parameter in parameter_change.iter_mut() {
                    let _ = parameter.take();
                }

                clone
            });

            // Lock the subscriber handler. This should not be allowed to be locked already, as the
            // changes are supposed to be made before using the database
            match self.subscriber_handler.try_lock() {
                None => Err(DatabaseError::SubscriberLock),
                Some(lock) => {
                    lock.borrow().notify_subscribers(self, &parameter_change);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn with_subscriber_handler<Function, ReturnType>(&self, f: Function) -> ReturnType
    where
        Function: FnOnce(&mut InternalSubscriberHandler) -> ReturnType,
    {
        let lock = self.subscriber_handler.lock();
        let mut inner = lock.borrow_mut();
        f(&mut inner)
    }
}
