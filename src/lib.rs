//! # Database
//!
//! A "database" in this context is a structure of parameters shared in a system. Parameters can be
//! subscribed to by any entity implementing the `DatabaseSubscriber` trait for a specific subset
//! of parameters in the database. Parameters can then be set which will notify all subscribers
//! that are subscribed to the specified references. These systems are built to be compile-time
//! verified and runs under a `no_std` codebase.
//!
//! Note that the database utilizes a combination of critical section mutex locks and spin locks.
//! This is so that a one core system can write to the database during different context switching
//! levels without the risk of dead locks. The notifying to subscribers should only be made in one
//! context as this requires a spin lock to not disable interrupts during subscribers notify calls.
//! The notify function will fail if its spin lock is enabled, as it is only intended for
//! subscribers to be added before any variables are set in the database

#![no_std]

mod content;
mod database;
mod database_error;
mod subscriber_handler;
mod tests;

pub use crate::{content::*, database::*, database_error::*, subscriber_handler::*};
