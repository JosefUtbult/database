use database::{DatabaseSubscriber, Subset};

use crate::database_content::*;

// `BobAndDebbieSubset` is a subset of members that exists in the `MyDatabaseContent` struct
#[derive(Subset, Default, Clone, Copy)]
#[superset(MyDatabase)]
pub struct BobAndDebbieSubset {
    bob: u16,
    debbie: bool,
}

pub struct BobAndDebbieSubscriber {}

impl DatabaseSubscriber<BobAndDebbieSubset, MyDatabaseMember, MY_DATABASE_MEMBER_COUNT>
    for BobAndDebbieSubscriber
{
    fn on_set(&self, change: &BobAndDebbieSubset) {
        std::println!(
            "Bob changed! Bob: {}, Debbie: {}",
            change.bob,
            change.debbie
        );
    }
}
