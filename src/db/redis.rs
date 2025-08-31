use redis::{AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let result: Option<String> = con.get(key).await?;
        Ok(result.and_then(|s| serde_json::from_str(&s).ok()))
    }

    pub async fn set<T: Serialize + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: usize,
    ) -> Result<(), RedisError> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let json_value = serde_json::to_string(value).unwrap();
        let _: () = con.set_ex(key, json_value, ttl_seconds as u64).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let _: i64 = con.del(key).await?;
        Ok(())
    }

    pub async fn get_and_delete<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RedisError> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;

        let (result, _): (Option<String>, i64) = redis::pipe()
            .get(key)
            .del(key)
            .query_async(&mut con)
            .await?;

        Ok(result.and_then(|s| serde_json::from_str(&s).ok()))
    }
}