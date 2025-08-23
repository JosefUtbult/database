#[derive(Debug, Clone, Copy)]
pub enum DatabaseError {
    SubscriberOverflow,
    SubscriberLock,
}
