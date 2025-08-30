use redis::{AsyncCommands, Client, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: DeserializeOwned,
    {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let result: Option<String> = con.get(key).await?;

        match result {
            Some(s) => Ok(serde_json::from_str(&s).ok()),
            None => Ok(None),
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: usize) -> Result<(), RedisError>
    where
        T: Serialize + Sync,
    {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let json_value = serde_json::to_string(value).unwrap();
        let _: () = con.set_ex(key, json_value, ttl_seconds as u64).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let _: () = con.del(key).await?;
        Ok(())
    }
}