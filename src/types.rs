use std::collections::HashMap;
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
}
