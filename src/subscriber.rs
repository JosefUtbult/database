use core::{
    cell::UnsafeCell,
    hash::Hash,
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

use heapless::{Vec, index_map::FnvIndexMap};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::{mutex::ScopedLocked, subscriber};

pub trait Subscriber<Keys> {
    fn on_change(&self, parameter_changes: &[Keys]);
}

type SubscriberIndex = u8;
pub const SUBSCRIBER_MAX_COUNT: usize = SubscriberIndex::MAX as usize + 1;

#[derive(Debug, PartialEq, Eq)]
pub enum SubscriberError {
    Locked,
    KeyError,
    Overflow,
}

struct InternalMutable<Key, const PARAMETER_COUNT: usize> {
    has_changed: Vec<Key, PARAMETER_COUNT>,
    key_to_subscriber_map:
        FnvIndexMap<Key, Vec<SubscriberIndex, SUBSCRIBER_MAX_COUNT>, PARAMETER_COUNT>,
    subscriber_to_key_map:
        FnvIndexMap<SubscriberIndex, Vec<Key, PARAMETER_COUNT>, SUBSCRIBER_MAX_COUNT>,
}

pub(crate) struct SubscriberData<'a, Mutex, Key, const PARAMETER_COUNT: usize>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: Ord + Hash + Copy,
{
    data: ScopedLocked<Mutex, InternalMutable<Key, PARAMETER_COUNT>>,
    subscribers: UnsafeCell<Vec<&'a dyn Subscriber<Key>, SUBSCRIBER_MAX_COUNT>>,
    allow_subscribers: AtomicBool,
    has_changed: AtomicBool,
}

impl<Key, const PARAMETER_COUNT: usize> InternalMutable<Key, PARAMETER_COUNT>
where
    Key: Ord + Hash + Copy,
{
    const fn new() -> Self {
        Self {
            has_changed: Vec::new(),
            key_to_subscriber_map: FnvIndexMap::new(),
            subscriber_to_key_map: FnvIndexMap::new(),
        }
    }
}

impl<'a, Mutex, Key, const PARAMETER_COUNT: usize> SubscriberData<'a, Mutex, Key, PARAMETER_COUNT>
where
    Mutex: ScopedRawMutex + ConstInit,
    Key: Ord + Hash + Copy,
{
    pub(crate) const fn new() -> Self {
        Self {
            subscribers: UnsafeCell::new(Vec::new()),
            data: ScopedLocked::new(InternalMutable::new()),
            allow_subscribers: AtomicBool::new(true),
            has_changed: AtomicBool::new(false),
        }
    }

    pub(crate) fn subscribe(
        &self,
        subscriber: &'a dyn Subscriber<Key>,
        key: Key,
    ) -> Result<(), SubscriberError> {
        // Check if subscribing is allowed or if parameter changes already started to occur
        if self.allow_subscribers.load(Ordering::SeqCst) {
            // Try to push a new subscriber to the subscriber list
            let subscriber_list: &mut Vec<&'a dyn Subscriber<Key>, SUBSCRIBER_MAX_COUNT> =
                unsafe { &mut *self.subscribers.get() };

            match subscriber_list.push(subscriber) {
                Err(_) => Err(SubscriberError::Overflow),
                Ok(_) => {
                    // Retrieve the new subscribers index
                    let subscriber_index = (subscriber_list.len() - 1) as SubscriberIndex;
                    let res = self.data.try_with(|data| {
                        // Get or create the map from subscribers -> subscribed key
                        let subscriber_to_key_vector = data
                            .subscriber_to_key_map
                            .entry(subscriber_index as u8)
                            .or_insert(Vec::new());

                        // Get or create the map from subscribed key -> subscriber
                        let key_to_subscriber_vector =
                            data.key_to_subscriber_map.entry(key).or_insert(Vec::new());

                        // Verify that both was created so that no mismatch occurred
                        if let Ok(subscriber_to_key_vector) = subscriber_to_key_vector
                            && let Ok(key_to_subscriber_vector) = key_to_subscriber_vector
                        {
                            // Push the key to the subscribers vector of subscribed keys
                            if !subscriber_to_key_vector.contains(&key) {
                                unsafe { subscriber_to_key_vector.push_unchecked(key) };
                            }

                            // Push the subscriber to the keys vector of subscribers
                            if !key_to_subscriber_vector.contains(&subscriber_index) {
                                unsafe {
                                    key_to_subscriber_vector.push_unchecked(subscriber_index)
                                };
                            }

                            Ok(())
                        } else {
                            Err(SubscriberError::KeyError)
                        }
                    });

                    match res {
                        Some(res) => res,
                        None => Err(SubscriberError::Locked),
                    }
                }
            }
        } else {
            Err(SubscriberError::Locked)
        }
    }

    pub(crate) fn on_change(&mut self, key: Key) {
        // Disallow further subscribers
        self.allow_subscribers.store(false, Ordering::SeqCst);

        // Set the has changed flag. This is used so that notify_subscribers doesn't have
        // to unlock the mutable data to check if data.has_changed is empty
        self.has_changed.store(true, Ordering::SeqCst);

        // Push the key to the has changed vector
        self.data.with(|data| {
            if !data.has_changed.contains(&key) {
                unsafe {
                    let _ = data.has_changed.push_unchecked(key);
                }
            }
        })
    }

    pub(crate) fn notify_subscribers(&self) {
        let subscribers_to_notify = self.data.with(|data| {
            // Go through each changed key. Collect all relevant subscribers
            let mut subscribers_to_notify: Vec<SubscriberIndex, SUBSCRIBER_MAX_COUNT> = Vec::new();
            for key in data.has_changed.iter() {
                let key_to_subscriber_vector = data
                    .key_to_subscriber_map
                    .entry(*key)
                    .or_insert(Vec::new())
                    .unwrap();

                for subscriber in key_to_subscriber_vector.iter() {
                    if !subscribers_to_notify.contains(subscriber) {
                        unsafe {
                            subscribers_to_notify.push_unchecked(*subscriber);
                        }
                    }
                }
            }

            subscribers_to_notify
        });

        for subscriber_index in subscribers_to_notify.iter() {
            let changed_parameters = self.data.with(|data| {
                // Retrieve all parameters the parameters relevant to the subscriber
                let subscriber_to_key_vector = data
                    .subscriber_to_key_map
                    .entry(*subscriber_index as u8)
                    .or_insert(Vec::new());

                match subscriber_to_key_vector {
                    Err(_) => panic!("Unable to get subscriber_to_key_vector"),
                    Ok(subscriber_to_key_vector) => {
                        // Collect all changed subscribed parameters
                        let mut changed_parameters: Vec<Key, PARAMETER_COUNT> = Vec::new();
                        for key in subscriber_to_key_vector.iter() {
                            if data.has_changed.contains(key) {
                                unsafe { changed_parameters.push_unchecked(*key) };
                            }
                        }

                        changed_parameters
                    }
                }
            });

            // Find the actual subscriber reference
            let subscriber_list = unsafe { &*self.subscribers.get() };
            let subscriber_index: usize = *subscriber_index as usize;
            assert!(subscriber_index < subscriber_list.len());

            // Notify the subscriber
            let subscriber = subscriber_list.get(subscriber_index).unwrap();
            subscriber.on_change(&changed_parameters);
        }
    }
}

#[cfg(test)]
mod test {
    use core::{
        panic,
        sync::atomic::{AtomicBool, Ordering},
    };

    use heapless::Vec;

    use crate::{
        Subscriber, SubscriberData,
        mutex::test_mutex::Mutex,
        test_types::test_types::{MY_DATA_PARAMETER_COUNT, MyDataKeys},
    };

    struct MySubscriber {
        was_notified: AtomicBool,
        subscribed_params: Vec<MyDataKeys, 3>,
    }

    impl MySubscriber {
        const fn new() -> Self {
            Self {
                was_notified: AtomicBool::new(false),
                subscribed_params: Vec::new(),
            }
        }
    }

    impl Subscriber<MyDataKeys> for MySubscriber {
        fn on_change(&self, parameter_changes: &[MyDataKeys]) {
            for parameter in parameter_changes {
                if !self.subscribed_params.contains(parameter) {
                    panic!("Got unsubscribed parameter {:?}", parameter);
                }
            }

            self.was_notified.store(true, Ordering::SeqCst);
        }
    }

    type MySubscriberData<'a> = SubscriberData<'a, Mutex, MyDataKeys, MY_DATA_PARAMETER_COUNT>;

    #[test]
    fn create_subscriber_data() {
        let _ = MySubscriberData::new();
    }

    #[test]
    fn subscribe() {
        let my_subscriber = MySubscriber::new();

        let subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataKeys::Param1)
            .unwrap();
    }

    #[test]
    fn notify_subscriber() {
        let mut my_subscriber = MySubscriber::new();
        my_subscriber.subscribed_params.push(MyDataKeys::Param1).unwrap();

        let mut subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataKeys::Param1)
            .unwrap();

        subscriber_data.on_change(MyDataKeys::Param1);
        subscriber_data.notify_subscribers();

        assert!(my_subscriber.was_notified.load(Ordering::SeqCst));
    }

    #[test]
    fn ignore_subscriber() {
        let mut my_subscriber = MySubscriber::new();
        my_subscriber.subscribed_params.push(MyDataKeys::Param1).unwrap();

        let mut subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataKeys::Param1)
            .unwrap();

        subscriber_data.on_change(MyDataKeys::Param2);
        subscriber_data.notify_subscribers();

        assert!(!my_subscriber.was_notified.load(Ordering::SeqCst));
    }
}
