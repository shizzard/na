//! Library crate containing REST API service code.
//!
//! Used by binary target and by integration tests.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unused_results
)]

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
#[allow(missing_docs)]
pub mod schema;

use diesel::{r2d2::ConnectionManager, PgConnection};
/// Database pool datatype.
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
