pub mod pool;
pub use pool::DbPool;

use async_trait::async_trait;
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    SurrealDB(String),
    Serialization(String),
    InvalidKey(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurrealDB(message) => write!(f, "SurrealDB error: {message}"),
            Self::Serialization(message) => write!(f, "Serialization error: {message}"),
            Self::InvalidKey(message) => write!(f, "Invalid key: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<surrealdb::Error> for Error {
    fn from(value: surrealdb::Error) -> Self {
        Self::SurrealDB(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait KVStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Value>>;
    async fn set(&self, key: &str, value: Value) -> Result<()>;
    async fn increment(&self, key: &str, delta: i64) -> Result<i64>;
}

#[cfg(test)]
mod tests {
    use super::{DbPool, KVStore};
    use serde_json::json;

    #[tokio::test]
    async fn get_set_and_increment_work_with_in_memory_db() {
        let store = DbPool::new_in_memory("test_ns", "test_db")
            .await
            .expect("in-memory SurrealDB should initialize");

        store
            .set("counter:global_home_visits", json!(7))
            .await
            .expect("set should store a numeric value");

        let value = store
            .get("counter:global_home_visits")
            .await
            .expect("get should read the value");
        assert_eq!(value, Some(json!(7)));

        let next = store
            .increment("counter:global_home_visits", 3)
            .await
            .expect("increment should update atomically");
        assert_eq!(next, 10);

        let updated = store
            .get("counter:global_home_visits")
            .await
            .expect("get should return the incremented value");
        assert_eq!(updated, Some(json!(10)));
    }

    #[tokio::test]
    async fn increment_creates_missing_counter_and_keeps_going() {
        let store = DbPool::new_in_memory("test_ns", "test_db")
            .await
            .expect("in-memory SurrealDB should initialize");

        let first = store
            .increment("counter:global_home_visits", 1)
            .await
            .expect("increment should create a first value");
        assert_eq!(first, 1);

        let second = store
            .increment("counter:global_home_visits", 1)
            .await
            .expect("increment should update the stored value");
        assert_eq!(second, 2);

        let third = store
            .increment("counter:global_home_visits", 1)
            .await
            .expect("increment should keep increasing");
        assert_eq!(third, 3);
    }

    #[tokio::test]
    async fn uuid_and_session_keys_work_with_in_memory_db() {
        let store = DbPool::new_in_memory("test_ns", "test_db")
            .await
            .expect("in-memory SurrealDB should initialize");

        let session_id = uuid::Uuid::new_v4().to_string();
        let key = format!("session:{session_id}");
        let user_data = json!({
            "id": "12345",
            "email": "test@example.com",
            "name": "Test User",
        });

        store
            .set(&key, user_data.clone())
            .await
            .expect("set should handle keys with UUIDs");

        let retrieved = store
            .get(&key)
            .await
            .expect("get should read keys with UUIDs");

        assert_eq!(retrieved, Some(user_data));
    }
}
