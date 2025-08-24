use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    Subset,
    content::DatabaseContent,
    database::{DatabaseHandler, DatabaseRef, ParameterChangeList},
    database_error::DatabaseError,
    subscriber_handler::{DatabaseSubscriber, DatabaseSubscriberHandler},
};

#[derive(Default, Clone, Copy)]
struct MyDatabaseContent {
    alice: u8,
    bob: u16,
    debbie: isize,
}

#[derive(Clone, Copy)]
struct MyContentSubset1 {
    alice: u8,
    debbie: isize,
}

#[derive(Clone, Copy)]
struct MyContentSubset2 {
    debbie: isize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyDatabaseMember {
    Alice(u8),
    Bob(u16),
    Debbie(isize),
}

impl From<MyDatabaseMember> for usize {
    fn from(value: MyDatabaseMember) -> Self {
        match value {
            MyDatabaseMember::Alice(_) => 0,
            MyDatabaseMember::Bob(_) => 1,
            MyDatabaseMember::Debbie(_) => 2,
        }
    }
}

struct MySubscriberHandler<'a> {
    my_content_subset1_subscribers:
        [Option<&'a dyn DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, 3>>; 128],

    my_content_subset2_subscribers:
        [Option<&'a dyn DatabaseSubscriber<MyContentSubset2, MyDatabaseMember, 3>>; 128],
}

impl<'a> MySubscriberHandler<'a> {
    const fn new() -> Self {
        Self {
            my_content_subset1_subscribers: [None; 128],
            my_content_subset2_subscribers: [None; 128],
        }
    }

    fn subscribe_with_my_content_subset1(
        &mut self,
        subscriber: &'a dyn DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, 3>,
    ) -> Result<(), DatabaseError> {
        for instance in self.my_content_subset1_subscribers.iter_mut() {
            if instance.is_none() {
                let _ = instance.insert(subscriber);
                return Ok(());
            }
        }
        Err(DatabaseError::SubscriberOverflow)
    }

    fn subscribe_with_my_content_subset2(
        &mut self,
        subscriber: &'a dyn DatabaseSubscriber<MyContentSubset2, MyDatabaseMember, 3>,
    ) -> Result<(), DatabaseError> {
        for instance in self.my_content_subset2_subscribers.iter_mut() {
            if instance.is_none() {
                let _ = instance.insert(subscriber);
                return Ok(());
            }
        }
        Err(DatabaseError::SubscriberOverflow)
    }
}

impl<'a> DatabaseSubscriberHandler<'a, MyDatabaseContent, MyDatabaseMember, 3>
    for MySubscriberHandler<'a>
{
    fn notify_subscribers(
        &self,
        database: &dyn DatabaseRef<MyDatabaseMember>,
        parameter_change: &ParameterChangeList<MyDatabaseMember, 3>,
    ) {
        // MyContentSubset1
        {
            if MyContentSubset1::is_subscribed(parameter_change) {
                let subset = MyContentSubset1::build_from_database(database);
                for instance in self.my_content_subset1_subscribers.iter() {
                    if let Some(instance) = instance {
                        instance.on_set(&subset);
                    }
                }
            }
        }
        // MyContentSubset2
        {
            if MyContentSubset2::is_subscribed(parameter_change) {
                let subset = MyContentSubset2::build_from_database(database);
                for instance in self.my_content_subset2_subscribers.iter() {
                    if let Some(instance) = instance {
                        instance.on_set(&subset);
                    }
                }
            }
        }
    }
}

impl MyDatabaseContent {
    fn new() -> Self {
        Self {
            alice: 13,
            bob: 4443,
            debbie: -1,
        }
    }
}

impl DatabaseContent<MyDatabaseMember, 3> for MyDatabaseContent {
    fn set(&mut self, parameter: MyDatabaseMember) {
        match parameter {
            MyDatabaseMember::Alice(value) => self.alice = value,
            MyDatabaseMember::Bob(value) => self.bob = value,
            MyDatabaseMember::Debbie(value) => self.debbie = value,
        }
    }

    fn get(&self, parameter: &MyDatabaseMember) -> MyDatabaseMember {
        match parameter {
            MyDatabaseMember::Alice(_) => MyDatabaseMember::Alice(self.alice),
            MyDatabaseMember::Bob(_) => MyDatabaseMember::Bob(self.bob),
            MyDatabaseMember::Debbie(_) => MyDatabaseMember::Debbie(self.debbie),
        }
    }
}

impl Subset<MyDatabaseMember, 3> for MyContentSubset1 {
    fn is_subscribed(parameter_change: &ParameterChangeList<MyDatabaseMember, 3>) -> bool {
        let alice_index: usize = MyDatabaseMember::Alice(u8::default()).into();
        let debbie_index: usize = MyDatabaseMember::Alice(u8::default()).into();
        parameter_change[alice_index].is_some() || parameter_change[debbie_index].is_some()
    }

    fn build_from_database(database: &dyn DatabaseRef<MyDatabaseMember>) -> Self {
        let alice = match database.internal_get(&MyDatabaseMember::Alice(u8::default())) {
            MyDatabaseMember::Alice(value) => value,
            _ => unreachable!(),
        };

        let debbie = match database.internal_get(&MyDatabaseMember::Debbie(isize::default())) {
            MyDatabaseMember::Debbie(value) => value,
            _ => unreachable!(),
        };

        Self { alice, debbie }
    }
}

impl Subset<MyDatabaseMember, 3> for MyContentSubset2 {
    fn is_subscribed(parameter_change: &ParameterChangeList<MyDatabaseMember, 3>) -> bool {
        let debbie_index: usize = MyDatabaseMember::Debbie(isize::default()).into();
        parameter_change[debbie_index].is_some()
    }

    fn build_from_database(database: &dyn DatabaseRef<MyDatabaseMember>) -> Self {
        let debbie = match database.internal_get(&MyDatabaseMember::Debbie(isize::default())) {
            MyDatabaseMember::Debbie(value) => value,
            _ => unreachable!(),
        };

        Self { debbie }
    }
}

fn build_database<'a>()
-> DatabaseHandler<'a, MyDatabaseContent, MySubscriberHandler<'a>, MyDatabaseMember, 3> {
    let database: DatabaseHandler<MyDatabaseContent, MySubscriberHandler, MyDatabaseMember, 3> =
        DatabaseHandler::new(MyDatabaseContent::new(), MySubscriberHandler::new());
    database
}

#[test]
fn simple_test() {
    let database = build_database();

    struct MySubsetSubscriber {}

    static HAS_TRIGGERED: AtomicBool = AtomicBool::new(false);
    impl DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, 3> for MySubsetSubscriber {
        fn on_set(&self, change: &MyContentSubset1) {
            HAS_TRIGGERED.store(true, Ordering::SeqCst);
            assert_eq!(change.alice, 2);
            assert_eq!(change.debbie, -1);
        }
    }

    let subscriber = MySubsetSubscriber {};

    database
        .get_subscriber_handler()
        .lock()
        .borrow_mut()
        .subscribe_with_my_content_subset1(&subscriber)
        .unwrap();

    let changes: [MyDatabaseMember; 2] = [MyDatabaseMember::Alice(2), MyDatabaseMember::Bob(144)];

    database.multi_set(&changes);
    database.notify_subscribers().unwrap();
    assert!(HAS_TRIGGERED.load(Ordering::SeqCst));
}

#[test]
fn dont_notify_non_subscribed() {
    let database = build_database();

    struct MySubsetSubscriber {}

    static HAS_TRIGGERED: AtomicBool = AtomicBool::new(false);
    impl DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, 3> for MySubsetSubscriber {
        fn on_set(&self, _change: &MyContentSubset1) {
            HAS_TRIGGERED.store(true, Ordering::SeqCst);
        }
    }

    let subscriber = MySubsetSubscriber {};

    database
        .get_subscriber_handler()
        .lock()
        .borrow_mut()
        .subscribe_with_my_content_subset1(&subscriber)
        .unwrap();

    database.set(&MyDatabaseMember::Bob(2));
    database.notify_subscribers().unwrap();
    assert!(!HAS_TRIGGERED.load(Ordering::SeqCst));
}

#[test]
fn multi_subscriber_test() {
    let database = build_database();

    struct MySubsetSubscriber1 {}
    struct MySubsetSubscriber2 {}

    static HAS_TRIGGERED1: AtomicBool = AtomicBool::new(false);
    static HAS_TRIGGERED2: AtomicBool = AtomicBool::new(false);

    impl DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, 3> for MySubsetSubscriber1 {
        fn on_set(&self, change: &MyContentSubset1) {
            HAS_TRIGGERED1.store(true, Ordering::SeqCst);
            assert_eq!(change.alice, 2);
            assert_eq!(change.debbie, -1);
        }
    }

    impl DatabaseSubscriber<MyContentSubset2, MyDatabaseMember, 3> for MySubsetSubscriber2 {
        fn on_set(&self, change: &MyContentSubset2) {
            HAS_TRIGGERED2.store(true, Ordering::SeqCst);
            assert_eq!(change.debbie, -1);
        }
    }

    let subscriber1 = MySubsetSubscriber1 {};
    let subscriber2 = MySubsetSubscriber2 {};

    {
        let handler_lock = database.get_subscriber_handler().lock();
        let mut handler = handler_lock.borrow_mut();

        handler
            .subscribe_with_my_content_subset1(&subscriber1)
            .unwrap();
        handler
            .subscribe_with_my_content_subset2(&subscriber2)
            .unwrap();
    }

    let changes: [MyDatabaseMember; 2] = [MyDatabaseMember::Alice(2), MyDatabaseMember::Bob(144)];

    database.multi_set(&changes);
    database.notify_subscribers().unwrap();
    assert!(HAS_TRIGGERED1.load(Ordering::SeqCst));
    assert!(!HAS_TRIGGERED2.load(Ordering::SeqCst));
}
