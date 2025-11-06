//! Postgres Unit of Work Module
//!
//! This module provides transaction handling primitives for PostgreSQL database operations.
//! It isolates transaction management from specific repository implementations.

pub mod executor;
pub mod transaction_aware;
pub mod unit_of_work;

pub use executor::Executor;
pub use transaction_aware::{TransactionAware, TransactionError, TransactionResult};
pub use unit_of_work::{UnitOfWork, UnitOfWorkSession, PostgresUnitOfWork, PostgresUnitOfWorkSession};
