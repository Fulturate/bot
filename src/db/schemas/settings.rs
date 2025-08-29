use crate::db::schemas::SettingsRepo;
use crate::util::errors::MyError;
use async_trait::async_trait;
use mongodb::bson::{doc, oid::ObjectId};
use oximod::{Model, ModelTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[db("fulturate")]
#[collection("settings")]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    _id: Option<ObjectId>,

    #[index(unique, name = "owner")]
    pub owner_id: String,

    pub owner_type: String,

    #[serde(default)]
    pub modules: Vec<ModuleSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSettings {
    pub key: String,             // уникальный ключ модуля, например "currency" или "speech_recog"
    pub enabled: bool,           // включен или нет
    pub description: String,     // описание модуля
    #[serde(default)]            // тут ещё было бы неплохо добавить лимиты, по типу максимум и какой сейчас,
                                 // но мне че то лень это продумывать
    pub options: Vec<ModuleOption>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleOption {
    pub key: String,
    pub value: String,
}

#[async_trait]
impl SettingsRepo for Settings {
    async fn get_or_create(owner_id: &str, owner_type: &str) -> Result<Self, MyError> {
        if let Some(found) = Settings::find_one(doc! { "owner_id": owner_id, "owner_type": owner_type }).await? {
            Ok(found)
        } else {
            let default_modules = vec![
                ModuleSettings {
                    key: "currency".to_string(),
                    enabled: false,
                    description: "Конвертация валют".to_string(),
                    options: vec![
                        ModuleOption { key: "currencies".into(), value: "USD,EUR".into() },
                    ],
                },
                ModuleSettings {
                    key: "speech".to_string(),
                    enabled: false,
                    description: "Распознавание речи".to_string(),
                    options: vec![
                        ModuleOption { key: "model".into(), value: "Gemini 2.5 Flash".into() },
                        ModuleOption { key: "token".into(), value: "".into() },
                    ],
                },
            ];
            let new = Settings::new()
                .owner_id(owner_id.to_string())
                .owner_type(owner_type.to_string())
                .modules(default_modules);
            ModelTrait::save(&new).await?;
            Settings::get_or_create(owner_id, owner_type).await
        }
    }

    fn modules(&self) -> &Vec<ModuleSettings> {
        &self.modules
    }

    fn modules_mut(&mut self) -> &mut Vec<ModuleSettings> {
        &mut self.modules
    }
}
