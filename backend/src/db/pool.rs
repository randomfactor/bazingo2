use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use surrealdb::engine::local::{Db as LocalDb, Mem, RocksDb};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use super::{Error, KVStore, Result};

#[derive(Clone)]
pub struct DbPool<T: surrealdb::Connection = Client> {
    inner: Arc<Surreal<T>>,
}

impl DbPool<Client> {
    pub async fn new(
        endpoint: &str,
        namespace: &str,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let db = Surreal::new::<Ws>(endpoint).await?;
        db.signin(Root {
            username,
            password,
        })
        .await?;
        db.use_ns(namespace).use_db(database).await?;

        Ok(Self {
            inner: Arc::new(db),
        })
    }

}

impl DbPool<LocalDb> {
    pub async fn from_env() -> Result<Self> {
        let path = std::env::var("SURREALDB_PATH").unwrap_or_else(|_| "./data/surrealdb".to_string());
        let namespace = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "main".to_string());
        let database = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "main".to_string());
        let username = std::env::var("SURREALDB_USER").unwrap_or_else(|_| "rskel_local".to_string());
        let password = std::env::var("SURREALDB_PASS").unwrap_or_else(|_| "local_only_duh".to_string());

        Self::new_local_rocksdb(&path, &namespace, &database, &username, &password).await
    }

    pub async fn new_in_memory(namespace: &str, database: &str) -> Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(namespace).use_db(database).await?;

        Ok(Self {
            inner: Arc::new(db),
        })
    }

    pub async fn new_local_rocksdb(
        path: &str,
        namespace: &str,
        database: &str,
        _username: &str,
        _password: &str,
    ) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path).await?;
        db.use_ns(namespace).use_db(database).await?;

        Ok(Self {
            inner: Arc::new(db),
        })
    }
}

fn split_key(key: &str) -> (String, String) {
    if let Some((table, id)) = key.split_once(':') {
        (table.to_string(), id.to_string())
    } else {
        ("kv".to_string(), key.to_string())
    }
}

#[async_trait]
impl<T: surrealdb::Connection> KVStore for DbPool<T> {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        if key.trim().is_empty() {
            return Err(Error::InvalidKey("key cannot be empty".to_string()));
        }

        let (table, id) = split_key(key);
        let mut response = self
            .inner
            .query("SELECT VALUE value FROM type::thing($tb, $id)")
            .bind(("tb", table))
            .bind(("id", id))
            .await?;

        let value: Option<Value> = response.take(0)?;
        Ok(value)
    }

    async fn set(&self, key: &str, value: Value) -> Result<()> {
        if key.trim().is_empty() {
            return Err(Error::InvalidKey("key cannot be empty".to_string()));
        }

        let (table, id) = split_key(key);
        self.inner
            .query("UPSERT type::thing($tb, $id) CONTENT { value: $value }")
            .bind(("tb", table))
            .bind(("id", id))
            .bind(("value", value))
            .await?;

        Ok(())
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        if key.trim().is_empty() {
            return Err(Error::InvalidKey("key cannot be empty".to_string()));
        }

        let key = key.to_string();
        let current = match self.get(&key).await? {
            Some(Value::Number(number)) => number.as_i64().unwrap_or(0),
            Some(Value::String(text)) => text.parse::<i64>().unwrap_or(0),
            Some(_) | None => 0,
        };

        let next = current + delta;
        self.set(&key, serde_json::json!(next)).await?;
        Ok(next)
    }
}
