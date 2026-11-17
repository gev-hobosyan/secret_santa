use std::collections::HashMap;
use sea_orm::{Database, DatabaseConnection};
use crate::types::ActiveGame;

#[derive(Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub active_games: HashMap<i64, ActiveGame>,
    pub langs: HashMap<i64, Lang>
}

#[derive(Clone, Copy, Debug)]
pub enum Lang {
    AM,
    EN
}

impl From<&String> for Lang {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "am" => Lang::AM,
            _ => Lang::EN
        }
    }
}

impl From<Lang> for String {
    fn from(value: Lang) -> Self {
        match value {
            Lang::AM => "am".to_string(),
            Lang::EN => "en".to_string()
        }
    }
}

impl AppState {
    pub async fn new() -> Self {
        let db = Database::connect("sqlite://app.sqlite?mode=rwc").await.unwrap();
        let active_games = HashMap::new();
        let langs = HashMap::new();
        
        Self {
            db,
            active_games,
            langs
        }
    }
}