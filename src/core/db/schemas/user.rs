use crate::core::{
    db::schemas::{BaseFunctions, CurrenciesFunctions},
    services::currency::converter::CurrencyStruct,
};
use async_trait::async_trait;
use mongodb::{
    bson,
    bson::{doc, oid::ObjectId},
    results::UpdateResult,
};
use oximod::{_error::oximod_error::OxiModError, Model};
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

    #[serde(default)]
    pub download_count: i64,
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

#[async_trait]
impl CurrenciesFunctions for User {
    fn get_id(&self) -> &str {
        &self.user_id
    }

    fn get_currencies(&self) -> &Vec<CurrencyStruct> {
        &self.convertable_currencies
    }

    async fn add_currency(
        user_id: &str,
        currency: &CurrencyStruct,
    ) -> Result<UpdateResult, OxiModError> {
        let currency_to_add = bson::to_bson(currency).unwrap();

        Self::update_one(
            doc! {"user_id": user_id},
            doc! {"$push": {"convertable_currencies": currency_to_add } },
        )
        .await
    }

    async fn remove_currency(user_id: &str, currency: &str) -> Result<UpdateResult, OxiModError> {
        Self::update_one(
            doc! {"user_id": user_id},
            doc! {"$pull": {"convertable_currencies": {"code": currency} } },
        )
        .await
    }
}

impl User {
    pub async fn get_or_create(id: &str) -> Result<bool, OxiModError> {
        if Self::find_one(doc! { "user_id": id }).await?.is_some() {
            Ok(false)
        } else {
            Self::new().user_id(id.to_string()).save().await?;
            Ok(true)
        }
    }

    pub async fn increment_download_count(user_id: &str) -> Result<UpdateResult, OxiModError> {
        Self::update_one(
            doc! { "user_id": user_id },
            doc! { "$inc": { "download_count": 1 } },
        )
        .await
    }
}
