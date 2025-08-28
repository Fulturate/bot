use crate::db::schemas::{BaseFunctions, CurrenciesFunctions, CurrencyStruct};
use async_trait::async_trait;
use mongodb::bson;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::results::UpdateResult;
use oximod::_error::oximod_error::OxiModError;
use oximod::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Model)]
#[db("fulturate")]
#[collection("groups")]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<ObjectId>,

    #[index(unique, name = "group_id")]
    pub group_id: String,

    #[index(sparse, name = "convertable_currencies")]
    #[serde(default)]
    pub convertable_currencies: Vec<CurrencyStruct>,
}

#[async_trait]
impl BaseFunctions for Group {
    async fn find_by_id(id: String) -> Result<Option<Self>, OxiModError> {
        Self::find_one(doc! { "group_id": id }).await
    }

    async fn create_with_id(id: String) -> Result<Self, OxiModError> {
        let new_group = Self::new()
            .group_id(id.clone())
            .convertable_currencies(vec![]);
        new_group.save().await?;
        
        <Group as BaseFunctions>::find_by_id(id)
            .await?
            .ok_or_else(|| {
                OxiModError::IndexError("Failed to retrieve newly created group".to_string())
            })
    }
}

#[async_trait]
impl CurrenciesFunctions for Group {
    fn get_id(&self) -> &str {
        &self.group_id
    }

    fn get_currencies(&self) -> &Vec<CurrencyStruct> {
        &self.convertable_currencies
    }

    async fn add_currency(
        group_id: &str,
        currency: &CurrencyStruct,
    ) -> Result<UpdateResult, OxiModError> {
        let currency_to_add = bson::to_bson(currency).unwrap();
        Self::update_one(
            doc! {"group_id": group_id},
            doc! {"$push": {"convertable_currencies": currency_to_add } },
        )
        .await
    }

    async fn remove_currency(group_id: &str, currency: &str) -> Result<UpdateResult, OxiModError> {
        Self::update_one(
            doc! {"group_id": group_id},
            doc! { "$pull": { "convertable_currencies": { "code": currency } } },
        )
        .await
    }
}
