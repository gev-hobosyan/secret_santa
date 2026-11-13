use std::fs;
use serde_json::Value;

pub struct Translate {
    messages: Value,
}

impl Translate {
    pub fn init(file_path: &str) -> std::io::Result<Self> {
        let file = fs::read_to_string(file_path)?;
        let messages: Value = serde_json::from_str(&file)?;

        Ok(Self {
            messages
        })
    }

    pub fn get(&self, lang: &str, key: &str) -> String {
        match self.messages[lang][key].clone() {
            Value::String(msg) => { msg }
            _ => { "".to_string() }
        }
    }

    pub fn with_args(&self, lang: &str, key: &str, args: Vec<&str>) -> String {
        match self.messages[lang][key].clone() {
            Value::String(msg) => {
                let mut res = msg;
                for arg in args {
                    res = res.replacen("{}", arg, 1);
                }

                res
            }
            _ => { "".to_string() }
        }
    }
}