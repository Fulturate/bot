use crate::db::schemas::{BaseFunctions, CurrenciesFunctions, CurrencyStruct};
use async_trait::async_trait;
use mongodb::bson::{doc, oid::ObjectId};
use oximod::{Model, _error::oximod_error::OxiModError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[db("fulturate")]
#[collection("users")]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<ObjectId>,
    #[index(unique, name = "user_id")]
    pub user_id: String,
    #[index(sparse, name = "convertable_currencies")]
    #[serde(default)]
    pub convertable_currencies: Vec<CurrencyStruct>,
}

#[async_trait]
impl BaseFunctions for User {
    async fn find_by_id(id: String) -> Result<Option<Self>, OxiModError> {
        Self::find_one(doc! { "user_id": id }).await
    }

    async fn create_with_id(id: String) -> Result<Self, OxiModError> {
        let new_user = Self::new()
            .user_id(id.clone())
            .convertable_currencies(vec![]);
        new_user.save().await?;
        <User as BaseFunctions>::find_by_id(id)
            .await?
            .ok_or_else(|| {
                OxiModError::IndexError("Failed to retrieve newly created user".to_string())
            })
    }
}

impl CurrenciesFunctions for User {
    fn get_id(&self) -> &str {
        &self.user_id
    }

    fn get_currencies(&self) -> &Vec<CurrencyStruct> {
        &self.convertable_currencies
    }

    fn id_field_name() -> &'static str {
        "user_id"
    }
}
