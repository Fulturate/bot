use redis::{AsyncCommands, Client, FromRedisValue, RedisError, ToRedisArgs};
use serde::Serialize;
use serde::de::DeserializeOwned;

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
        T: FromRedisValue + DeserializeOwned,
    {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let result: Option<T> = con.get(key).await?;
        Ok(result)
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: usize) -> Result<(), RedisError>
    where
        T: ToRedisArgs + Serialize + Sync,
    {
        let mut con = self.client.get_multiplexed_tokio_connection().await?;
        let _: () = con.set_ex(key, value, ttl_seconds as u64).await?;
        Ok(())
    }
}
