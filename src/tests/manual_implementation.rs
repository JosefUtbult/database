use core::sync::atomic::{AtomicBool, Ordering};

use fixed_string::FixedString;

use crate::{
    Subset,
    content::DatabaseContent,
    database::{DatabaseHandler, DatabaseRef, ParameterChangeList},
    database_error::DatabaseError,
    subscriber_handler::{DatabaseSubscriber, DatabaseSubscriberHandler},
};

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
struct MyDatabaseContent {
    alice: u8,
    bob: u16,
    charlie: FixedString<20>,
    debbie: isize,
}

#[derive(Clone, Copy)]
struct MyContentSubset {
    alice: u8,
    debbie: isize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyDatabaseParameters {
    Alice(u8),
    Bob(u16),
    Charlie(FixedString<20>),
    Debbie(isize),
}

impl From<MyDatabaseParameters> for usize {
    fn from(value: MyDatabaseParameters) -> Self {
        match value {
            MyDatabaseParameters::Alice(_) => 0,
            MyDatabaseParameters::Bob(_) => 1,
            MyDatabaseParameters::Charlie(_) => 2,
            MyDatabaseParameters::Debbie(_) => 3,
        }
    }
}

struct MySubscriberHandler<'a> {
    my_content_subset_subscribers:
        [Option<&'a dyn DatabaseSubscriber<MyContentSubset, MyDatabaseParameters, 3>>; 128],
}

impl<'a> MySubscriberHandler<'a> {
    const fn new() -> Self {
        Self {
            my_content_subset_subscribers: [None; 128],
        }
    }

    fn subscribe_with_content_subset(
        &mut self,
        subscriber: &'a dyn DatabaseSubscriber<MyContentSubset, MyDatabaseParameters, 3>,
    ) -> Result<(), DatabaseError> {
        for instance in self.my_content_subset_subscribers.iter_mut() {
            if instance.is_none() {
                let _ = instance.insert(subscriber);
                return Ok(());
            }
        }
        Err(DatabaseError::SubscriberOverflow)
    }
}

impl<'a> DatabaseSubscriberHandler<MyDatabaseContent, MyDatabaseParameters, 3>
    for MySubscriberHandler<'a>
{
    fn notify_subscribers(
        &self,
        database: &dyn DatabaseRef<MyDatabaseParameters>,
        parameter_change: &ParameterChangeList<MyDatabaseParameters, 3>,
    ) {
        // MyContentSubset
        {
            if MyContentSubset::is_subscribed(parameter_change) {
                let subset = MyContentSubset::build_from_database(database);
                for instance in self.my_content_subset_subscribers.iter() {
                    if let Some(instance) = instance {
                        instance.on_set(&subset);
                    }
                }
            }
        }
    }
}

impl MyDatabaseContent {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            alice: 13,
            bob: 4443,
            charlie: FixedString::new_with("Hello").unwrap(),
            debbie: -1,
        }
    }
}

impl DatabaseContent<MyDatabaseParameters, 3> for MyDatabaseContent {
    fn set(&mut self, parameter: MyDatabaseParameters) {
        match parameter {
            MyDatabaseParameters::Alice(value) => self.alice = value,
            MyDatabaseParameters::Bob(value) => self.bob = value,
            MyDatabaseParameters::Charlie(value) => self.charlie = value,
            MyDatabaseParameters::Debbie(value) => self.debbie = value,
        }
    }

    fn get(&self, parameter: &MyDatabaseParameters) -> MyDatabaseParameters {
        match parameter {
            MyDatabaseParameters::Alice(_) => MyDatabaseParameters::Alice(self.alice),
            MyDatabaseParameters::Bob(_) => MyDatabaseParameters::Bob(self.bob),
            MyDatabaseParameters::Charlie(_) => MyDatabaseParameters::Charlie(self.charlie),
            MyDatabaseParameters::Debbie(_) => MyDatabaseParameters::Debbie(self.debbie),
        }
    }
}

impl Subset<MyDatabaseParameters, 3> for MyContentSubset {
    fn is_subscribed(parameter_change: &ParameterChangeList<MyDatabaseParameters, 3>) -> bool {
        let alice_index: usize = MyDatabaseParameters::Alice(u8::default()).into();
        let debbie_index: usize = MyDatabaseParameters::Alice(u8::default()).into();
        parameter_change[alice_index].is_some() || parameter_change[debbie_index].is_some()
    }

    fn build_from_database(database: &dyn DatabaseRef<MyDatabaseParameters>) -> Self {
        let alice = match database.internal_get(&MyDatabaseParameters::Alice(u8::default())) {
            MyDatabaseParameters::Alice(value) => value,
            _ => unreachable!(),
        };

        let debbie = match database.internal_get(&MyDatabaseParameters::Debbie(isize::default())) {
            MyDatabaseParameters::Debbie(value) => value,
            _ => unreachable!(),
        };

        Self { alice, debbie }
    }
}

#[test]
fn test() {
    let database: DatabaseHandler<MyDatabaseContent, MySubscriberHandler, MyDatabaseParameters, 3> =
        DatabaseHandler::new(MyDatabaseContent::new(), MySubscriberHandler::new());

    struct MySubsetSubscriber {}

    static HAS_TRIGGERED: AtomicBool = AtomicBool::new(false);
    impl DatabaseSubscriber<MyContentSubset, MyDatabaseParameters, 3> for MySubsetSubscriber {
        fn on_set(&self, change: &MyContentSubset) {
            HAS_TRIGGERED.store(true, Ordering::SeqCst);
            assert_eq!(change.alice, 2);
            assert_eq!(change.debbie, -1);
        }
    }

    let subscriber = MySubsetSubscriber {};

    database
        .with_subscriber_handler(|subscriber_handler| {
            subscriber_handler.subscribe_with_content_subset(&subscriber)
        })
        .unwrap();

    let changes: [MyDatabaseParameters; 2] = [
        MyDatabaseParameters::Alice(2),
        MyDatabaseParameters::Bob(144),
    ];

    database.multi_set(&changes);
    database.notify_subscribers().unwrap();
    assert!(HAS_TRIGGERED.load(Ordering::SeqCst));
}
