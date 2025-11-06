use async_trait::async_trait;
use parking_lot::RwLock;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

use crate::{Executor, TransactionAware, TransactionResult};

/// Unit of Work pattern for managing database transactions.
///
/// The UnitOfWork manages the lifecycle of database transactions and provides
/// a factory method to create new transaction sessions.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    type Session: UnitOfWorkSession;
    
    /// Begin a new transaction session.
    async fn begin(&self) -> TransactionResult<Self::Session>;
}

/// Represents a single database transaction session.
///
/// This trait provides the core transaction management operations and a
/// mechanism to register transaction-aware components that need to be
/// notified of transaction lifecycle events.
#[async_trait]
pub trait UnitOfWorkSession: Send + Sync {
    /// Get the executor for this session (provides access to the transaction).
    fn executor(&self) -> &Executor;
    
    /// Register a component that needs to be notified of transaction events.
    fn register_transaction_aware(&self, observer: Arc<dyn TransactionAware>);
    
    /// Commit the transaction and notify all registered observers.
    async fn commit(self) -> TransactionResult<()>;
    
    /// Rollback the transaction and notify all registered observers.
    async fn rollback(self) -> TransactionResult<()>;
}

/// Default implementation of UnitOfWork for PostgreSQL.
pub struct PostgresUnitOfWork {
    pool: Arc<PgPool>,
}

impl PostgresUnitOfWork {
    /// Create a new PostgresUnitOfWork with the given connection pool.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    type Session = PostgresUnitOfWorkSession;
    
    async fn begin(&self) -> TransactionResult<Self::Session> {
        let tx = self.pool.begin().await?;
        Ok(PostgresUnitOfWorkSession::new(tx))
    }
}

/// Default implementation of UnitOfWorkSession for PostgreSQL.
pub struct PostgresUnitOfWorkSession {
    executor: Executor,
    observers: Arc<RwLock<Vec<Arc<dyn TransactionAware>>>>,
}

impl PostgresUnitOfWorkSession {
    /// Create a new session from a PostgreSQL transaction.
    pub fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self {
            executor: Executor::new(tx),
            observers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl UnitOfWorkSession for PostgresUnitOfWorkSession {
    fn executor(&self) -> &Executor {
        &self.executor
    }
    
    fn register_transaction_aware(&self, observer: Arc<dyn TransactionAware>) {
        self.observers.write().push(observer);
    }
    
    async fn commit(self) -> TransactionResult<()> {
        // Take ownership of the transaction
        let tx = self.executor.take_transaction().await?;
        
        // Commit the transaction
        tx.commit().await?;
        
        // Notify observers after successful commit
        let observers = self.observers.read().clone();
        for observer in observers.iter() {
            observer.on_commit().await?;
        }
        Ok(())
    }
    
    async fn rollback(self) -> TransactionResult<()> {
        // Take ownership of the transaction
        let tx = self.executor.take_transaction().await?;
        
        // Rollback the transaction
        tx.rollback().await?;
        
        // Notify observers after successful rollback
        let observers = self.observers.read().clone();
        for observer in observers.iter() {
            observer.on_rollback().await?;
        }
        Ok(())
    }
}