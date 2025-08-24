use crate::{
    alice_and_bob_subscriber::AliceAndBobSubscriber,
    bob_and_debbie_subscriber::BobAndDebbieSubscriber,
    database_content::{MyDatabase, MyDatabaseContent, MyDatabaseMember},
};

mod alice_and_bob_subscriber;
mod bob_and_debbie_subscriber;
mod database_content;
mod res;

fn main() {
    // Create a database object
    let database = MyDatabase::new(MyDatabaseContent::new());

    // Add subscribers to the database. Note that these are subset-specific; the `AliceAndBobSubscriber`
    // subscriber can only subscribe with the generated `subscribe_with_alice_and_bob_subset`
    // function
    let alice_and_bob_subscriber = AliceAndBobSubscriber {};
    let bob_and_debbie_subscriber = BobAndDebbieSubscriber {};
    {
        let subscriber_handler_lock = database.get_subscriber_handler().lock();
        let mut subscriber_handler = subscriber_handler_lock.borrow_mut();

        subscriber_handler
            .subscribe_with_alice_and_bob_subset(&alice_and_bob_subscriber)
            .unwrap();

        subscriber_handler
            .subscribe_with_bob_and_debbie_subset(&bob_and_debbie_subscriber)
            .unwrap();
    }

    // Create a list of changes. These are on the form of the generated `MyDatabaseMember` enum,
    // with the new values supplied
    let changes: [MyDatabaseMember; 2] =
        [MyDatabaseMember::Alice(2), MyDatabaseMember::Charlie(-1)];

    // Apply the changes and notify all subscribers. This only notifies the
    // `AliceAndBobSubscriber`, as the `BobAndDebbieSubset` doesn't contain any members that where
    // changed
    database.multi_set(&changes);
    database.notify_subscribers().unwrap();
}
