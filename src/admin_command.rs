use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use crate::app_state::AppState;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule="lowercase")]
pub enum AdminCommand {
    Start,
    Cancel,
    Finish
}

pub async fn handle_admin_commands(bot: Bot, msg: Message, cmd: AdminCommand, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
    if !is_admin(&bot, &msg).await? {
        return Ok(())
    }
    
    let state_clone = Arc::clone(&state);
    let state_clone = state_clone.lock().await;
    let db = &state_clone.db;

    let state = state.lock().await;
    
    match cmd {
        AdminCommand::Start => {}
        AdminCommand::Cancel => {}
        AdminCommand::Finish => {}
    }

    Ok(())
}

async fn is_admin(bot: &Bot, msg: &Message) -> ResponseResult<bool> {
    let Some(user) = &msg.from else { return Ok(false) };

    let admins = bot.get_chat_administrators(msg.chat.id).await?;

    Ok(admins.iter().any(|admin| admin.user.id == user.id))
}