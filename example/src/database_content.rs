use database::Database;

use crate::alice_and_bob_subscriber::AliceAndBobSubset;
use crate::bob_and_debbie_subscriber::BobAndDebbieSubset;

/// Declares a database content struct. This is all parameters that will be contained in the
/// database. A database struct type called `MyDatabase` is then constructed using the derive
/// Database macro. This macro will also generate
/// - the enum `MyDatabaseMember` containing an enum
///   variant of all members in the struct
/// - a `MY_DATABASE_MEMBER_COUNT` usize specifying the
///   number of members in this enum
/// - a `MyDatabaseSubscriberHandler` that handles all subsets to this database and their
///   subscribers
#[derive(Database, Copy, Clone, Default)]
#[name(MyDatabase)] // The resulting name of the database struct will be `MyDatabase`
#[subset(AliceAndBobSubset)] // `AliceAndBobSubset` is a struct with the members `alice` and `bob`
#[subset(BobAndDebbieSubset)] // `BobAndDebbieSubset` is a struct with the members `bob` and `debbie`
pub struct MyDatabaseContent {
    alice: u8,
    bob: u16,
    charlie: isize,
    debbie: bool,
}

impl MyDatabaseContent {
    pub const fn new() -> Self {
        Self {
            alice: 4,
            bob: 5333,
            charlie: -1,
            debbie: true,
        }
    }
}
