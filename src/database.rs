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

/// A list of all parameters in the parameter space. These are set to some when a parameter has
/// changed. Note that the `Parameter` type should be an enum created automatically by the
/// `Database` proc-macro
pub type ParameterChangeList<Parameter, const PARAMETER_COUNT: usize> =
    [Option<Parameter>; PARAMETER_COUNT];

/// Internal implementation of a database reference. This is used as the Database type cannot be
/// templated in the subscriber handler using the subscriber handler itself, as this isn't
/// compile-time calculable
pub trait DatabaseRef<Parameter>
where
    Parameter: Clone + Copy + Eq,
{
    /// Gives the same result as `get`
    fn internal_get(&self, parameter: &Parameter) -> Parameter;
}

/// A `Database` structure is a component that keeps track of an internal content list of
/// parameters, a list of subscriber and whether parameters has changed
pub struct DatabaseHandler<
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
    for DatabaseHandler<InternalContent, InternalSubscriberHandler, Parameter, PARAMETER_COUNT>
where
    Parameter: Copy + Clone + Eq,
    usize: From<Parameter>,
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
    InternalSubscriberHandler:
        DatabaseSubscriberHandler<InternalContent, Parameter, PARAMETER_COUNT>,
{
    /// Glue to get the database to be referenced by a subscriber handler
    fn internal_get(&self, parameter: &Parameter) -> Parameter {
        self.get(parameter)
    }
}

impl<InternalContent, InternalSubscriberHandler, Parameter, const PARAMETER_COUNT: usize>
    DatabaseHandler<InternalContent, InternalSubscriberHandler, Parameter, PARAMETER_COUNT>
where
    Parameter: Copy + Clone + Eq,
    usize: From<Parameter>,
    InternalContent: DatabaseContent<Parameter, PARAMETER_COUNT>,
    InternalSubscriberHandler:
        DatabaseSubscriberHandler<InternalContent, Parameter, PARAMETER_COUNT>,
{
    /// Create a new instance if a `Database`, templated with the content, subscriber handler,
    /// parameter enum type and the number of members in that enum
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

    /// Retrieve a value from the database
    pub fn get(&self, parameter: &Parameter) -> Parameter {
        critical(|cs| {
            let internal = self.content.borrow(cs).borrow();
            internal.get(parameter)
        })
    }

    /// Set an array of parameters in a database. This will store a changed state for the provided
    /// parameters, which later is acted upon by calling the `notify_subscribers` function
    pub fn multi_set(&self, parameters: &[Parameter]) {
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

    /// Set a parameter in a database. This will store a changed state for the provided
    /// parameter, which later is acted upon by calling the `notify_subscribers` function
    pub fn set(&self, parameter: &Parameter) {
        let list = [parameter.clone(); 1];
        self.multi_set(&list);
    }

    /// Notify all subscribers of changes made to the database. This is separated out from the set
    /// functionality, as these might need to run under different contexts/priority levels. This
    /// function presumes that no other entity is actively handling the list of internal
    /// subscribers. If the internal subscribers are locked for any reason, this will cause a
    /// `DatabaseError`
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
                Some(lock) => {
                    lock.borrow().notify_subscribers(self, &parameter_change);
                    Ok(())
                }
                None => {
                    // Put back the last known state of the notify list
                    critical(|cs| {
                        let mut internal_change_list = self.change_list.borrow(cs).borrow_mut();
                        for (internal_parameter, taken_parameter) in
                            internal_change_list.iter_mut().zip(parameter_change.iter())
                        {
                            if let Some(taken_parameter) = taken_parameter {
                                // If the internal parameter hasn't been changed, as in if anyone
                                // else has set it, put the original value back
                                if internal_parameter.is_none() {
                                    let _ = internal_parameter.insert(*taken_parameter);
                                }
                            }
                        }
                    });
                    Err(DatabaseError::SubscriberLock)
                }
            }
        } else {
            Ok(())
        }
    }

    /// Retrieve a handle to the internal subscriber handler. Used to subscribe to different
    /// subsets of the parameter space. This should be done before actively using the database, as
    /// this can cause locking errors resulting in a failure to notify subscribers
    pub fn with_subscriber_handler<Function, ReturnType>(&self, f: Function) -> ReturnType
    where
        Function: FnOnce(&mut InternalSubscriberHandler) -> ReturnType,
    {
        let lock = self.subscriber_handler.lock();
        let mut inner = lock.borrow_mut();
        f(&mut inner)
    }
}
