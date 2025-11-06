use async_trait::async_trait;

/// Error type for transaction-aware operations
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Transaction commit failed: {0}")]
    CommitFailed(String),
    
    #[error("Transaction rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Result type for transaction-aware operations
pub type TransactionResult<T> = Result<T, TransactionError>;

/// Trait for components that need to be notified of transaction lifecycle events.
///
/// Components implementing this trait can be registered with a UnitOfWorkSession
/// to receive callbacks when the transaction is committed or rolled back.
/// This allows repositories and other components to perform cleanup operations,
/// update caches, or handle other post-transaction tasks.
#[async_trait]
pub trait TransactionAware: Send + Sync {
    /// Called after a successful transaction commit.
    ///
    /// Implementations should use this to finalize any pending operations,
    /// such as updating caches or flushing buffers.
    async fn on_commit(&self) -> TransactionResult<()>;
    
    /// Called after a transaction rollback.
    ///
    /// Implementations should use this to revert any in-memory state changes
    /// that were made during the transaction.
    async fn on_rollback(&self) -> TransactionResult<()>;
}