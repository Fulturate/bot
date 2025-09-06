use redis::{AsyncCommands, Client, RedisError};
use serde::{Serialize, de::DeserializeOwned};

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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub async fn set_url_hash_mapping(
        &self,
        url_hash: &str,
        original_url: &str,
        ttl_seconds: usize,
    ) -> Result<(), RedisError> {
        let key = format!("url_hash:{}", url_hash);
        self.set(&key, &original_url.to_string(), ttl_seconds).await
    }

    // todo: remove them on pre-release stage
    #[allow(dead_code)]
    pub async fn get_url_by_hash(&self, url_hash: &str) -> Result<Option<String>, RedisError> {
        let key = format!("url_hash:{}", url_hash);
        self.get(&key).await
    }
}
