use std::collections::HashMap;
use sea_orm::Iden;
use teloxide::types::MessageId;

#[derive(Debug)]
pub struct ActiveGame {
    pub message_id: MessageId,
    pub players: HashMap<i64, String>
}

impl ActiveGame {
    pub fn new(message_id: MessageId) -> Self {
        Self {
            message_id,
            players: HashMap::new()
        }
    }
    
    pub fn format_players(&self) -> String {
        let mut players = "".to_string();
        
        for (_, username) in &self.players {
            players += &format!("@{}, ", username);
        }

        players.pop();
        players.pop();
        
        players
    }
}
