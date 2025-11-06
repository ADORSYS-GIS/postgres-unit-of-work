use uuid::Uuid;

/// Sample User entity for testing
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
        }
    }
}

/// Sample Order entity for testing
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_name: String,
    pub amount: i64,
}

impl Order {
    pub fn new(user_id: Uuid, product_name: String, amount: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            product_name,
            amount,
        }
    }
}