use mongodb::bson::{doc, oid::ObjectId};
use oximod::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrencyStruct {
    pub code: String,
    pub source: String,
    #[serde(default)]
    pub api_identifier: Option<String>,
    pub symbol: String,
    pub flag: String,
    pub patterns: Vec<String>,
    pub one: String,
    pub few: String,
    pub many: String,
    #[allow(dead_code)]
    pub one_en: String,
    #[allow(dead_code)]
    pub many_en: String,
    pub is_target: bool,
}

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
