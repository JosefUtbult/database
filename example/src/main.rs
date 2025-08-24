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
    let database = MyDatabase::new(MyDatabaseContent::new());

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

    let changes: [MyDatabaseMember; 2] =
        [MyDatabaseMember::Alice(2), MyDatabaseMember::Charlie(-1)];

    database.multi_set(&changes);
    database.notify_subscribers().unwrap();
}
