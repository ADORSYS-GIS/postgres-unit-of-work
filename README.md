# PostgreSQL Unit of Work

A robust implementation of the Unit of Work pattern for PostgreSQL database transactions in Rust.

## Features

- Transaction lifecycle management
- Transaction-aware repositories
- Support for commit/rollback operations
- Observer pattern for transaction events
- Thread-safe executor pattern

## Running Tests

### Prerequisites

- Docker and Docker Compose
- Rust toolchain

### Start Test Database

```bash
docker-compose up -d
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_commit_functionality
cargo test test_rollback_functionality
cargo test test_multiple_transactions_isolation
```

### Clean Database

```bash
# Stop and remove volumes (clean database)
docker-compose down -v
```

## Test Structure

The test suite includes:

1. **Sample Entities**: [`User`](tests/common/entities.rs:7) and [`Order`](tests/common/entities.rs:22) entities with UUID primary keys
2. **Transaction-Aware Repositories**: [`UserRepository`](tests/common/repositories.rs:12) and [`OrderRepository`](tests/common/repositories.rs:89) implementing the [`TransactionAware`](src/transaction_aware.rs:26) trait
3. **Comprehensive Tests**:
   - [`test_commit_functionality`](tests/unit_of_work_test.rs:63): Verifies data persists after commit
   - [`test_rollback_functionality`](tests/unit_of_work_test.rs:147): Verifies data is discarded after rollback
   - [`test_multiple_transactions_isolation`](tests/unit_of_work_test.rs:243): Tests transaction isolation

## Key Design Decisions

- **UUID Primary Keys**: Using random UUIDs avoids ID conflicts and counting issues in tests
- **Repeatable Tests**: Each test cleans up after itself; `docker-compose down -v` provides a clean slate
- **Transaction Callbacks**: Repositories track commit/rollback events for verification
- **Isolation Testing**: Multiple transactions can be tested independently

## Example Usage

```rust
use postgres_unit_of_work::{PostgresUnitOfWork, UnitOfWork, UnitOfWorkSession};
use std::sync::Arc;

// Initialize unit of work
let uow = PostgresUnitOfWork::new(Arc::new(pool));

// Begin transaction
let session = uow.begin().await?;

// Create repositories
let user_repo = UserRepository::new(session.executor().clone());
let order_repo = OrderRepository::new(session.executor().clone());

// Register transaction-aware components
session.register_transaction_aware(user_repo.clone());
session.register_transaction_aware(order_repo.clone());

// Perform operations
user_repo.create(&user).await?;
order_repo.create(&order).await?;

// Commit or rollback
session.commit().await?;
// or
session.rollback().await?;
```

## Database Configuration

Default connection string: `postgres://postgres:postgres@localhost:5432/test_db`

Override with environment variable:
```bash
export DATABASE_URL="postgres://user:password@host:port/database"