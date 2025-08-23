use core::sync::atomic::{AtomicBool, Ordering};

use fixed_string::FixedString;

use crate::{
    Database, DatabaseContent, DatabaseError, DatabaseRef, DatabaseSubscriber,
    DatabaseSubscriberHandler, ParameterChangeList,
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
    my_content_subset_subscribers: [Option<&'a dyn DatabaseSubscriber<MyContentSubset>>; 128],
}

impl<'a> MySubscriberHandler<'a> {
    const fn new() -> Self {
        Self {
            my_content_subset_subscribers: [None; 128],
        }
    }

    fn subscribe_with_content_subset(
        &mut self,
        subscriber: &'a dyn DatabaseSubscriber<MyContentSubset>,
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
        #[allow(unused_variables)]
        let alice = match database.internal_get(&MyDatabaseParameters::Alice(u8::default())) {
            MyDatabaseParameters::Alice(value) => value,
            _ => unreachable!(),
        };

        #[allow(unused_variables)]
        let bob = match database.internal_get(&MyDatabaseParameters::Bob(u16::default())) {
            MyDatabaseParameters::Bob(value) => value,
            _ => unreachable!(),
        };

        #[allow(unused_variables)]
        let charlie = match database
            .internal_get(&MyDatabaseParameters::Charlie(FixedString::<20>::default()))
        {
            MyDatabaseParameters::Charlie(value) => value,
            _ => unreachable!(),
        };

        #[allow(unused_variables)]
        let debbie = match database.internal_get(&MyDatabaseParameters::Debbie(isize::default())) {
            MyDatabaseParameters::Debbie(value) => value,
            _ => unreachable!(),
        };

        // MyContentSubset
        let index: usize = MyDatabaseParameters::Alice(u8::default()).into();
        if parameter_change[index].is_some()
            || parameter_change[MyDatabaseParameters::Charlie as usize].is_some()
        {
            let payload = MyContentSubset { alice, debbie };

            for subscriber in self.my_content_subset_subscribers {
                if let Some(subscriber) = subscriber {
                    subscriber.on_set(&payload);
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

#[test]
fn test() {
    let database: Database<MyDatabaseContent, MySubscriberHandler, MyDatabaseParameters, 3> =
        Database::new(MyDatabaseContent::new(), MySubscriberHandler::new());

    struct MySubsetSubscriber {}

    static HAS_TRIGGERED: AtomicBool = AtomicBool::new(false);
    impl DatabaseSubscriber<MyContentSubset> for MySubsetSubscriber {
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

    database.set(&changes);
    database.notify_subscribers().unwrap();
    assert!(HAS_TRIGGERED.load(Ordering::SeqCst));
}
