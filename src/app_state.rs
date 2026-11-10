use std::collections::HashMap;
use sea_orm::{Database, DatabaseConnection};
use crate::types::ActiveGame;

pub struct AppState {
    pub db: DatabaseConnection,
    pub active_games: HashMap<i64, ActiveGame>
}

impl AppState {
    pub async fn new() -> Self {
        let db = Database::connect("sqlite://secret.sqlite?mode=rwc").await.unwrap();
        let active_games = HashMap::new();
        
        Self {
            db,
            active_games
        }
    }
}