use crate::core::db::schemas::SettingsRepo;
use crate::errors::MyError;
use async_trait::async_trait;
use mongodb::bson;
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
    pub key: String,   // уникальный ключ модуля, например "currency" или "speech_recog"
    pub enabled: bool, // включен или нет
    pub description: String, // описание модуля
    #[serde(default)] // тут ещё было бы неплохо добавить лимиты, по типу максимум и какой сейчас,
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
        if let Some(found) =
            Settings::find_one(doc! { "owner_id": owner_id, "owner_type": owner_type }).await?
        {
            Ok(found)
        } else {
            let default_modules = vec![
                ModuleSettings {
                    key: "currency".to_string(),
                    enabled: false,
                    description: "Конвертация валют".to_string(),
                    options: vec![ModuleOption {
                        key: "currencies".into(),
                        value: "USD,EUR".into(),
                    }],
                },
                ModuleSettings {
                    key: "speech".to_string(),
                    enabled: false,
                    description: "Распознавание речи".to_string(),
                    options: vec![
                        ModuleOption {
                            key: "model".into(),
                            value: "Gemini 2.5 Flash".into(),
                        },
                        ModuleOption {
                            key: "token".into(),
                            value: "".into(),
                        },
                    ],
                },
                create_cobalt_module(),
            ];
            let new = Settings::new()
                .owner_id(owner_id.to_string())
                .owner_type(owner_type.to_string())
                .modules(default_modules);
            ModelTrait::save(&new).await?;
            Settings::get_or_create(owner_id, owner_type).await
        }
    }

    async fn update_module<F>(
        owner_id: &str,
        owner_type: &str,
        module_key: &str,
        modifier: F,
    ) -> Result<Self, MyError>
    where
        Self: Sized,
        F: FnOnce(&mut ModuleSettings) + Send,
    {
        let mut settings = Self::get_or_create(owner_id, owner_type).await?;

        if let Some(module) = settings
            .modules_mut()
            .iter_mut()
            .find(|m| m.key == module_key)
        {
            modifier(module);
        } else {
            return Err(MyError::ModuleNotFound(module_key.to_string()));
        }

        let filter = doc! { "owner_id": owner_id, "owner_type": owner_type };
        let modules_as_bson = bson::to_bson(&settings.modules)?;
        let update = doc! { "$set": { "modules": modules_as_bson } };

        Self::update_one(filter, update).await?;

        Ok(settings)
    }

    fn modules_mut(&mut self) -> &mut Vec<ModuleSettings> {
        &mut self.modules
    }
}

fn create_cobalt_module() -> ModuleSettings {
    ModuleSettings {
        key: "cobalt".to_string(),
        enabled: true,
        description: "Настройки для Cobalt Downloader".to_string(),
        options: vec![
            ModuleOption {
                key: "preferred_output".into(),
                value: "auto".into(),
            },
            ModuleOption {
                key: "video_format".into(),
                value: "h264".into(),
            },
            ModuleOption {
                key: "video_quality".into(),
                value: "1080".into(),
            },
            ModuleOption {
                key: "audio_format".into(),
                value: "mp3".into(),
            },
            ModuleOption {
                key: "attribution".into(),
                value: "false".into(),
            }, // Используем строку "false" для унификации
        ],
    }
}
