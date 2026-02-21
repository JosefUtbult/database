use core::{
    cell::UnsafeCell,
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

use heapless::{LinearMap, Vec};
use mutex_traits::{ConstInit, ScopedRawMutex};

use crate::mutex::ScopedLocked;

pub trait Subscriber<Keys> {
    fn on_change(&self, parameter_changes: &[Keys]);
}

type SubscriberIndex = u8;
pub const SUBSCRIBER_MAX_COUNT: usize = SubscriberIndex::MAX as usize;

#[derive(Debug, PartialEq, Eq)]
pub enum SubscriberError {
    Locked,
    KeyError,
    Overflow,
}

struct VectorMap<FlatKey, FlatField, const KEY_COUNT: usize, const PARAM_COUNT: usize>(
    LinearMap<FlatKey, Vec<FlatField, PARAM_COUNT>, KEY_COUNT>,
)
where
    FlatKey: Eq + Clone + Copy;

impl<FlatKey, FlatField, const KEY_COUNT: usize, const PARAM_COUNT: usize>
    VectorMap<FlatKey, FlatField, KEY_COUNT, PARAM_COUNT>
where
    FlatKey: Eq + Clone + Copy,
{
    const fn new() -> Self {
        Self(LinearMap::new())
    }

    fn get(&self, key: &FlatKey) -> Option<&Vec<FlatField, PARAM_COUNT>> {
        self.0.get(key)
    }

    pub fn get_or_create(&mut self, key: &FlatKey) -> &mut Vec<FlatField, PARAM_COUNT> {
        if !self.0.contains_key(key) {
            if self.0.insert(*key, Vec::new()).is_err() {
                panic!("Unable to insert into linear map");
            }
        }

        self.0.get_mut(key).unwrap()
    }
}

struct InternalMutable<FlatKey, const FLAT_PARAMETER_COUNT: usize>
where
    FlatKey: Eq + Clone + Copy,
{
    has_changed: Vec<FlatKey, FLAT_PARAMETER_COUNT>,
    key_to_subscriber_map:
        VectorMap<FlatKey, SubscriberIndex, FLAT_PARAMETER_COUNT, SUBSCRIBER_MAX_COUNT>,
    subscriber_to_key_map:
        VectorMap<SubscriberIndex, FlatKey, SUBSCRIBER_MAX_COUNT, FLAT_PARAMETER_COUNT>,
}

pub(crate) struct SubscriberData<'a, Mutex, FlatKey, const FLAT_PARAMETER_COUNT: usize>
where
    Mutex: ScopedRawMutex + ConstInit,
    FlatKey: Eq + Clone + Copy,
{
    data: ScopedLocked<Mutex, InternalMutable<FlatKey, FLAT_PARAMETER_COUNT>>,
    subscribers: UnsafeCell<Vec<&'a dyn Subscriber<FlatKey>, SUBSCRIBER_MAX_COUNT>>,
    allow_subscribers: AtomicBool,
    has_changed: AtomicBool,
}

impl<FlatKey, const FLAT_PARAMETER_COUNT: usize> InternalMutable<FlatKey, FLAT_PARAMETER_COUNT>
where
    FlatKey: Eq + Clone + Copy,
{
    const fn new() -> Self {
        Self {
            has_changed: Vec::new(),
            key_to_subscriber_map: VectorMap::new(),
            subscriber_to_key_map: VectorMap::new(),
        }
    }
}

impl<'a, Mutex, FlatKey, const FLAT_PARAMETER_COUNT: usize>
    SubscriberData<'a, Mutex, FlatKey, FLAT_PARAMETER_COUNT>
where
    Mutex: ScopedRawMutex + ConstInit,
    FlatKey: Eq + Clone + Copy,
{
    #[allow(dead_code)]
    pub(crate) const fn new() -> Self {
        Self {
            subscribers: UnsafeCell::new(Vec::new()),
            data: ScopedLocked::new(InternalMutable::new()),
            allow_subscribers: AtomicBool::new(true),
            has_changed: AtomicBool::new(false),
        }
    }

    fn add_subscriber_to_list(
        &self,
        subscriber: &'a dyn Subscriber<FlatKey>,
    ) -> Result<u8, SubscriberError> {
        // Try to push a new subscriber to the subscriber list
        let subscriber_list: &mut Vec<&'a dyn Subscriber<FlatKey>, SUBSCRIBER_MAX_COUNT> =
            unsafe { &mut *self.subscribers.get() };

        #[cfg(test)]
        std::println!("subscriber list len: {}", subscriber_list.len());

        // Check if the subscriber already exists
        let mut index: u8 = 0;
        for item in subscriber_list.iter() {
            if core::ptr::addr_eq(
                subscriber as *const dyn Subscriber<FlatKey>,
                *item as *const dyn Subscriber<FlatKey>,
            ) {
                return Ok(index);
            }
            index += 1;
        }

        #[cfg(test)]
        std::println!("Didn't find subscriber");

        // Push the subscriber and return the last index
        match subscriber_list.push(subscriber) {
            Err(_) => Err(SubscriberError::Overflow),
            Ok(_) => Ok((subscriber_list.len() - 1) as u8),
        }
    }

    fn map_subscriber_to_key(
        &self,
        subscriber_index: u8,
        key: FlatKey,
    ) -> Result<(), SubscriberError> {
        let res = self.data.try_with(|data| {
            // Get or create the map from subscribers -> subscribed key
            let subscriber_to_key_vector =
                data.subscriber_to_key_map.get_or_create(&subscriber_index);

            // Get or create the map from subscribed key -> subscriber
            let key_to_subscriber_vector = data.key_to_subscriber_map.get_or_create(&key);

            // Push the key to the subscribers vector of subscribed keys
            if !subscriber_to_key_vector.contains(&key) {
                unsafe { subscriber_to_key_vector.push_unchecked(key) };
            }

            // Push the subscriber to the keys vector of subscribers
            if !key_to_subscriber_vector.contains(&subscriber_index) {
                unsafe { key_to_subscriber_vector.push_unchecked(subscriber_index) };
            }

            Ok(())
        });

        match res {
            Some(res) => res,
            None => Err(SubscriberError::Locked),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn subscribe(
        &self,
        subscriber: &'a dyn Subscriber<FlatKey>,
        key: FlatKey,
    ) -> Result<(), SubscriberError> {
        // Check if subscribing is allowed or if parameter changes already started to occur
        if self.allow_subscribers.load(Ordering::SeqCst) {
            // Add and retrieve the subscriber index
            let subscriber_index = self.add_subscriber_to_list(subscriber)?;

            // Map the subscriber to and from the key
            self.map_subscriber_to_key(subscriber_index, key)
        } else {
            Err(SubscriberError::Locked)
        }
    }

    pub(crate) fn on_changes(&self, keys: &[FlatKey]) {
        // Disallow further subscribers
        self.allow_subscribers.store(false, Ordering::SeqCst);

        // Set the has changed flag. This is used so that notify_subscribers doesn't have
        // to unlock the mutable data to check if data.has_changed is empty
        self.has_changed.store(true, Ordering::SeqCst);

        // Push the key to the has changed vector
        self.data.with(|data| {
            for key in keys {
                if !data.has_changed.contains(&key) {
                    unsafe {
                        let _ = data.has_changed.push_unchecked(*key);
                    }
                }
            }
        })
    }

    pub(crate) fn on_change(&self, key: FlatKey) {
        self.on_changes(&[key])
    }

    fn collect_subscribers_to_notify(&self) -> Vec<SubscriberIndex, SUBSCRIBER_MAX_COUNT> {
        self.data.with(|data| {
            // Go through each changed key. Collect all relevant subscribers
            let mut subscribers_to_notify: Vec<SubscriberIndex, SUBSCRIBER_MAX_COUNT> = Vec::new();
            for key in data.has_changed.iter() {
                if let Some(key_to_subscriber_vector) = data.key_to_subscriber_map.get(key) {
                    for subscriber in key_to_subscriber_vector.iter() {
                        if !subscribers_to_notify.contains(subscriber) {
                            unsafe {
                                subscribers_to_notify.push_unchecked(*subscriber);
                            }
                        }
                    }
                }
            }

            subscribers_to_notify
        })
    }

    fn notify_single_subscriber(&self, subscriber_index: &u8) {
        let changed_parameters = self.data.with(|data| {
            // Retrieve all parameters the parameters relevant to the subscriber
            let subscriber_to_key_vector =
                data.subscriber_to_key_map.get(subscriber_index).unwrap();

            // Collect all changed subscribed parameters
            let mut changed_parameters: Vec<FlatKey, FLAT_PARAMETER_COUNT> = Vec::new();
            for key in subscriber_to_key_vector.iter() {
                if data.has_changed.contains(key) {
                    unsafe { changed_parameters.push_unchecked(*key) };
                }
            }

            changed_parameters
        });

        // Find the actual subscriber reference
        let subscriber_list = unsafe { &*self.subscribers.get() };
        let subscriber_index: usize = *subscriber_index as usize;
        assert!(subscriber_index < subscriber_list.len());

        // Notify the subscriber
        let subscriber = subscriber_list.get(subscriber_index).unwrap();
        subscriber.on_change(&changed_parameters);
    }

    #[allow(dead_code)]
    pub(crate) fn notify_subscribers(&self) {
        let subscribers_to_notify = self.collect_subscribers_to_notify();
        for subscriber_index in subscribers_to_notify.iter() {
            self.notify_single_subscriber(subscriber_index);
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
        SUBSCRIBER_MAX_COUNT, Subscriber, SubscriberData, SubscriberError,
        database_core::AllKeys,
        mutex::test_mutex::Mutex,
        test_types::test_types::{MY_DATA_PARAMETER_FLAT_COUNT, MyDataFlatKeys},
    };

    struct MySubscriber {
        was_notified: AtomicBool,
        subscribed_params: Vec<MyDataFlatKeys, 3>,
    }

    impl MySubscriber {
        const fn new() -> Self {
            Self {
                was_notified: AtomicBool::new(false),
                subscribed_params: Vec::new(),
            }
        }
    }

    impl Subscriber<MyDataFlatKeys> for MySubscriber {
        fn on_change(&self, parameter_changes: &[MyDataFlatKeys]) {
            for parameter in parameter_changes {
                if !self.subscribed_params.contains(parameter) {
                    panic!("Got unsubscribed parameter {:?}", parameter);
                }
            }

            self.was_notified.store(true, Ordering::SeqCst);
        }
    }

    type MySubscriberData<'a> =
        SubscriberData<'a, Mutex, MyDataFlatKeys, MY_DATA_PARAMETER_FLAT_COUNT>;

    #[test]
    fn create_subscriber_data() {
        let _ = MySubscriberData::new();
    }

    #[test]
    fn subscribe() {
        let my_subscriber = MySubscriber::new();

        let subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();
    }

    #[test]
    fn notify_subscriber() {
        let mut my_subscriber = MySubscriber::new();
        my_subscriber
            .subscribed_params
            .push(MyDataFlatKeys::Param1)
            .unwrap();

        let subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();

        subscriber_data.on_change(MyDataFlatKeys::Param1);
        subscriber_data.notify_subscribers();

        assert!(my_subscriber.was_notified.load(Ordering::SeqCst));
    }

    #[test]
    fn same_subscriber_on_same_key() {
        let my_subscriber = MySubscriber::new();

        let mut subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();

        assert_eq!(subscriber_data.subscribers.get_mut().len(), 1);

        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();

        assert_eq!(subscriber_data.subscribers.get_mut().len(), 1);

        subscriber_data.data.with(|data| {
            let key_to_subscriber_vec = data
                .key_to_subscriber_map
                .get(&MyDataFlatKeys::Param1)
                .unwrap();
            let subscriber_to_key_vec = data.subscriber_to_key_map.get(&0).unwrap();

            assert_eq!(key_to_subscriber_vec.len(), 1);
            assert_eq!(subscriber_to_key_vec.len(), 1);

            assert_eq!(key_to_subscriber_vec[0], 0);
            assert_eq!(subscriber_to_key_vec[0], MyDataFlatKeys::Param1);
        });
    }

    #[test]
    fn same_subscriber_on_different_keys() {
        let my_subscriber = MySubscriber::new();

        let mut subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();

        assert_eq!(subscriber_data.subscribers.get_mut().len(), 1);

        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param2)
            .unwrap();

        assert_eq!(subscriber_data.subscribers.get_mut().len(), 1);
    }

    #[test]
    fn ignore_subscriber() {
        let mut my_subscriber = MySubscriber::new();
        my_subscriber
            .subscribed_params
            .push(MyDataFlatKeys::Param1)
            .unwrap();

        let subscriber_data = MySubscriberData::new();
        subscriber_data
            .subscribe(&my_subscriber, MyDataFlatKeys::Param1)
            .unwrap();

        subscriber_data.on_change(MyDataFlatKeys::Param2);
        subscriber_data.notify_subscribers();

        assert!(!my_subscriber.was_notified.load(Ordering::SeqCst));
    }

    #[test]
    fn overflow_subscribers() {
        let my_subscribers: [MySubscriber; SUBSCRIBER_MAX_COUNT] =
            [const { MySubscriber::new() }; SUBSCRIBER_MAX_COUNT];

        let last_subscriber = MySubscriber::new();

        let subscriber_data = MySubscriberData::new();
        let mut counter = 0;
        for subscriber in my_subscribers.iter() {
            std::println!("Index {}", counter);
            counter += 1;

            for key in MyDataFlatKeys::ALL_KEYS.iter() {
                subscriber_data.subscribe(subscriber, *key).unwrap();
            }
        }

        let res = subscriber_data.subscribe(&last_subscriber, MyDataFlatKeys::Param1);
        assert!(matches!(res, Err(SubscriberError::Overflow)));
    }
}
