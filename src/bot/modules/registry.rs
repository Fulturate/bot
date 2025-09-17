use super::{Module, cobalt::CobaltModule};
use crate::bot::modules::currency::CurrencyModule;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, sync::Arc};

pub struct ModuleManager {
    modules: BTreeMap<String, Arc<dyn Module>>,
}

impl ModuleManager {
    fn new() -> Self {
        let modules: Vec<Arc<dyn Module>> = vec![Arc::new(CobaltModule), Arc::new(CurrencyModule)];

        let modules = modules
            .into_iter()
            .map(|module| (module.key().to_string(), module))
            .collect();

        Self { modules }
    }

    pub fn get_module(&self, key: &str) -> Option<&Arc<dyn Module>> {
        self.modules.get(key)
    }

    pub fn get_all_modules(&self) -> Vec<&Arc<dyn Module>> {
        self.modules.values().collect()
    }
}

pub static MOD_MANAGER: Lazy<ModuleManager> = Lazy::new(ModuleManager::new);
