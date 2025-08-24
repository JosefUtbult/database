use core::sync::atomic::{AtomicBool, Ordering};

use database_macro::Database;

use crate::{DatabaseSubscriber, Subset};

#[derive(Database, Default, Clone, Copy)]
#[name(MyDatabase)]
#[subset(MyContentSubset1)]
#[subset(MyContentSubset2)]
struct MyDatabaseContent {
    alice: u8,
    bob: u16,
    debbie: isize,
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

#[derive(Subset, Clone, Copy)]
#[superset(MyDatabase)]
struct MyContentSubset1 {
    alice: u8,
    debbie: isize,
}

#[derive(Subset, Clone, Copy)]
#[superset(MyDatabase)]
struct MyContentSubset2 {
    debbie: isize,
}

#[test]
fn simple_test() {
    let database = MyDatabase::new(MyDatabaseContent::new());
    struct MySubsetSubscriber {}

    static HAS_TRIGGERED: AtomicBool = AtomicBool::new(false);
    impl DatabaseSubscriber<MyContentSubset1, MyDatabaseMember, MY_DATABASE_MEMBER_COUNT>
        for MySubsetSubscriber
    {
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
    let database = MyDatabase::new(MyDatabaseContent::new());

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
    let database = MyDatabase::new(MyDatabaseContent::new());

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
