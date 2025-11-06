use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Executor wraps a database transaction for use by repositories.
///
/// This struct provides a shared reference to a PostgreSQL transaction
/// that can be passed to multiple repositories within a unit of work.
#[derive(Clone, Debug)]
pub struct Executor {
    pub tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl Executor {
    /// Creates a new Executor from a PostgreSQL transaction.
    pub fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self {
            tx: Arc::new(Mutex::new(Some(tx))),
        }
    }
    
    /// Takes ownership of the transaction, leaving None in its place.
    /// This should only be called when committing or rolling back.
    pub(crate) async fn take_transaction(&self) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
        self.tx.lock().await.take().ok_or(sqlx::Error::PoolClosed)
    }
}