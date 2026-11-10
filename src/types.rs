use std::collections::HashMap;
use teloxide::types::MessageId;

pub struct ActiveGame {
    message_id: MessageId,
    players: HashMap<i64, String>
}

