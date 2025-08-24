use database::{DatabaseSubscriber, Subset};

use crate::database_content::*;

#[derive(Subset, Default, Clone, Copy)]
#[superset(MyDatabase)]
pub struct AliceAndBobSubset {
    alice: u8,
    bob: u16,
}

pub struct AliceAndBobSubscriber {}

impl DatabaseSubscriber<AliceAndBobSubset, MyDatabaseMember, MY_DATABASE_MEMBER_COUNT>
    for AliceAndBobSubscriber
{
    fn on_set(&self, change: &AliceAndBobSubset) {
        std::println!(
            "Alice or Bob changed! Alice: {}, Bob: {}",
            change.alice,
            change.bob
        );
    }
}
