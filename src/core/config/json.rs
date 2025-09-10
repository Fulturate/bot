use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct JsonConfig {
    pub ai_model: String,
    pub ai_prompt: String,
    pub summarize_prompt: String,
}

impl JsonConfig {
    pub fn get_ai_model(&self) -> &str {
        &self.ai_model
    }

    pub fn get_ai_prompt(&self) -> &str {
        &self.ai_prompt
    }

    pub fn get_summarize_prompt(&self) -> &str {
        &self.summarize_prompt
    }
}

pub fn read_json_config<P: AsRef<Path>>(path: P) -> Result<JsonConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: JsonConfig = serde_json::from_str(&contents)?;
    Ok(config)
}
