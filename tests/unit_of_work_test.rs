mod common;

use postgres_unit_of_work::{PostgresUnitOfWork, UnitOfWork, UnitOfWorkSession};
use sqlx::PgPool;
use std::sync::Arc;

use common::{Order, OrderRepository, User, UserRepository};

/// Helper function to get database URL from environment or use default
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string())
}

/// Setup the database connection pool and create tables
async fn setup_database() -> PgPool {
    let pool = PgPool::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    // Create tables for testing
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id),
            product_name VARCHAR(255) NOT NULL,
            amount BIGINT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create orders table");

    pool
}

/// Clean up database after tests
async fn cleanup_database(pool: &PgPool) {
    sqlx::query("DROP TABLE IF EXISTS orders CASCADE")
        .execute(pool)
        .await
        .expect("Failed to drop orders table");

    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(pool)
        .await
        .expect("Failed to drop users table");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial_test::serial]
async fn test_commit_functionality() {
    // Setup
    let pool = setup_database().await;
    let uow = PostgresUnitOfWork::new(Arc::new(pool.clone()));

    // Create a new transaction session
    let session = uow.begin().await.expect("Failed to begin transaction");

    // Create repositories
    let user_repo = UserRepository::new(session.executor().clone());
    let order_repo = OrderRepository::new(session.executor().clone());

    // Register repositories as transaction-aware
    session.register_transaction_aware(user_repo.clone());
    session.register_transaction_aware(order_repo.clone());

    // Create test data
    let user = User::new("john_doe".to_string(), "john@example.com".to_string());
    let order = Order::new(user.id, "Laptop".to_string(), 1200);

    // Perform operations within the transaction
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    order_repo
        .create(&order)
        .await
        .expect("Failed to create order");

    // Verify data exists within transaction
    let found_user = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");
    assert_eq!(found_user.username, user.username);

    let found_order = order_repo
        .find_by_id(order.id)
        .await
        .expect("Failed to find order")
        .expect("Order not found");
    assert_eq!(found_order.product_name, order.product_name);

    // Commit the transaction
    session.commit().await.expect("Failed to commit transaction");

    // Verify transaction-aware callbacks were called
    assert!(user_repo.is_committed(), "User repository should be committed");
    assert!(order_repo.is_committed(), "Order repository should be committed");
    assert!(!user_repo.is_rolled_back(), "User repository should not be rolled back");
    assert!(!order_repo.is_rolled_back(), "Order repository should not be rolled back");

    // Verify data persists after commit in a new transaction
    let verify_session = uow.begin().await.expect("Failed to begin verify transaction");
    let verify_user_repo = UserRepository::new(verify_session.executor().clone());
    let verify_order_repo = OrderRepository::new(verify_session.executor().clone());

    let persisted_user = verify_user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find persisted user")
        .expect("Persisted user not found");
    assert_eq!(persisted_user.username, user.username);

    let persisted_order = verify_order_repo
        .find_by_id(order.id)
        .await
        .expect("Failed to find persisted order")
        .expect("Persisted order not found");
    assert_eq!(persisted_order.product_name, order.product_name);

    verify_session.commit().await.expect("Failed to commit verify transaction");

    // Cleanup
    cleanup_database(&pool).await;
    pool.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial_test::serial]
async fn test_rollback_functionality() {
    // Setup
    let pool = setup_database().await;
    let uow = PostgresUnitOfWork::new(Arc::new(pool.clone()));

    // Get initial counts
    let count_session = uow.begin().await.expect("Failed to begin count transaction");
    let count_user_repo = UserRepository::new(count_session.executor().clone());
    let count_order_repo = OrderRepository::new(count_session.executor().clone());
    
    let initial_user_count = count_user_repo.count().await.expect("Failed to count users");
    let initial_order_count = count_order_repo.count().await.expect("Failed to count orders");
    
    count_session.commit().await.expect("Failed to commit count transaction");

    // Create a new transaction session
    let session = uow.begin().await.expect("Failed to begin transaction");

    // Create repositories
    let user_repo = UserRepository::new(session.executor().clone());
    let order_repo = OrderRepository::new(session.executor().clone());

    // Register repositories as transaction-aware
    session.register_transaction_aware(user_repo.clone());
    session.register_transaction_aware(order_repo.clone());

    // Create test data
    let user = User::new("jane_doe".to_string(), "jane@example.com".to_string());
    let order = Order::new(user.id, "Smartphone".to_string(), 800);

    // Perform operations within the transaction
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    order_repo
        .create(&order)
        .await
        .expect("Failed to create order");

    // Verify data exists within transaction
    let found_user = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user");
    assert!(found_user.is_some(), "User should exist in transaction");

    let found_order = order_repo
        .find_by_id(order.id)
        .await
        .expect("Failed to find order");
    assert!(found_order.is_some(), "Order should exist in transaction");

    // Rollback the transaction
    session.rollback().await.expect("Failed to rollback transaction");

    // Verify transaction-aware callbacks were called
    assert!(!user_repo.is_committed(), "User repository should not be committed");
    assert!(!order_repo.is_committed(), "Order repository should not be committed");
    assert!(user_repo.is_rolled_back(), "User repository should be rolled back");
    assert!(order_repo.is_rolled_back(), "Order repository should be rolled back");

    // Verify data does NOT persist after rollback
    let verify_session = uow.begin().await.expect("Failed to begin verify transaction");
    let verify_user_repo = UserRepository::new(verify_session.executor().clone());
    let verify_order_repo = OrderRepository::new(verify_session.executor().clone());

    let not_found_user = verify_user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to query user");
    assert!(not_found_user.is_none(), "User should not exist after rollback");

    let not_found_order = verify_order_repo
        .find_by_id(order.id)
        .await
        .expect("Failed to query order");
    assert!(not_found_order.is_none(), "Order should not exist after rollback");

    // Verify counts remain unchanged
    let final_user_count = verify_user_repo.count().await.expect("Failed to count users");
    let final_order_count = verify_order_repo.count().await.expect("Failed to count orders");
    
    assert_eq!(final_user_count, initial_user_count, "User count should be unchanged");
    assert_eq!(final_order_count, initial_order_count, "Order count should be unchanged");

    verify_session.commit().await.expect("Failed to commit verify transaction");

    // Cleanup
    cleanup_database(&pool).await;
    pool.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial_test::serial]
async fn test_multiple_transactions_isolation() {
    // Setup
    let pool = setup_database().await;
    let uow = PostgresUnitOfWork::new(Arc::new(pool.clone()));

    // Transaction 1: Create and commit a user
    let session1 = uow.begin().await.expect("Failed to begin transaction 1");
    let user_repo1 = UserRepository::new(session1.executor().clone());
    session1.register_transaction_aware(user_repo1.clone());

    let user1 = User::new("alice".to_string(), "alice@example.com".to_string());
    user_repo1.create(&user1).await.expect("Failed to create user1");
    session1.commit().await.expect("Failed to commit transaction 1");

    // Transaction 2: Create but rollback another user
    let session2 = uow.begin().await.expect("Failed to begin transaction 2");
    let user_repo2 = UserRepository::new(session2.executor().clone());
    session2.register_transaction_aware(user_repo2.clone());

    let user2 = User::new("bob".to_string(), "bob@example.com".to_string());
    user_repo2.create(&user2).await.expect("Failed to create user2");
    session2.rollback().await.expect("Failed to rollback transaction 2");

    // Verify only user1 exists
    let verify_session = uow.begin().await.expect("Failed to begin verify transaction");
    let verify_user_repo = UserRepository::new(verify_session.executor().clone());

    let found_user1 = verify_user_repo
        .find_by_id(user1.id)
        .await
        .expect("Failed to find user1")
        .expect("User1 should exist");
    assert_eq!(found_user1.username, "alice");

    let not_found_user2 = verify_user_repo
        .find_by_id(user2.id)
        .await
        .expect("Failed to query user2");
    assert!(not_found_user2.is_none(), "User2 should not exist after rollback");

    verify_session.commit().await.expect("Failed to commit verify transaction");

    // Cleanup
    cleanup_database(&pool).await;
    pool.close().await;
}