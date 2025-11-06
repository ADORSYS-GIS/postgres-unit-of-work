use async_trait::async_trait;
use parking_lot::RwLock;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};

use super::entities::{Order, User};

/// Transaction-aware User Repository
pub struct UserRepository {
    executor: Executor,
    // Track operations for verification in tests
    committed: Arc<RwLock<bool>>,
    rolled_back: Arc<RwLock<bool>>,
}

impl UserRepository {
    pub fn new(executor: Executor) -> Arc<Self> {
        Arc::new(Self {
            executor,
            committed: Arc::new(RwLock::new(false)),
            rolled_back: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn create(&self, user: &User) -> TransactionResult<()> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        sqlx::query(
            "INSERT INTO users (id, username, email) VALUES ($1, $2, $3)"
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> TransactionResult<Option<User>> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        let row = sqlx::query(
            "SELECT id, username, email FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
        }))
    }

    pub async fn count(&self) -> TransactionResult<i64> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        let row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&mut **tx)
            .await?;
        Ok(row.get("count"))
    }

    pub fn is_committed(&self) -> bool {
        *self.committed.read()
    }

    pub fn is_rolled_back(&self) -> bool {
        *self.rolled_back.read()
    }
}

#[async_trait]
impl TransactionAware for UserRepository {
    async fn on_commit(&self) -> TransactionResult<()> {
        *self.committed.write() = true;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        *self.rolled_back.write() = true;
        Ok(())
    }
}

/// Transaction-aware Order Repository
pub struct OrderRepository {
    executor: Executor,
    // Track operations for verification in tests
    committed: Arc<RwLock<bool>>,
    rolled_back: Arc<RwLock<bool>>,
}

impl OrderRepository {
    pub fn new(executor: Executor) -> Arc<Self> {
        Arc::new(Self {
            executor,
            committed: Arc::new(RwLock::new(false)),
            rolled_back: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn create(&self, order: &Order) -> TransactionResult<()> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        sqlx::query(
            "INSERT INTO orders (id, user_id, product_name, amount) VALUES ($1, $2, $3, $4)"
        )
        .bind(order.id)
        .bind(order.user_id)
        .bind(&order.product_name)
        .bind(order.amount)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> TransactionResult<Option<Order>> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        let row = sqlx::query(
            "SELECT id, user_id, product_name, amount FROM orders WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;

        Ok(row.map(|r| Order {
            id: r.get("id"),
            user_id: r.get("user_id"),
            product_name: r.get("product_name"),
            amount: r.get("amount"),
        }))
    }

    pub async fn count(&self) -> TransactionResult<i64> {
        let mut tx_guard = self.executor.tx.lock().await;
        let tx = tx_guard.as_mut().ok_or(sqlx::Error::PoolClosed)?;
        let row = sqlx::query("SELECT COUNT(*) as count FROM orders")
            .fetch_one(&mut **tx)
            .await?;
        Ok(row.get("count"))
    }

    pub fn is_committed(&self) -> bool {
        *self.committed.read()
    }

    pub fn is_rolled_back(&self) -> bool {
        *self.rolled_back.read()
    }
}

#[async_trait]
impl TransactionAware for OrderRepository {
    async fn on_commit(&self) -> TransactionResult<()> {
        *self.committed.write() = true;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        *self.rolled_back.write() = true;
        Ok(())
    }
}