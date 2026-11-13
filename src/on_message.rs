use std::sync::Arc;
use sea_orm::Iden;
use sea_orm::Value::String;
use teloxide::Bot;
use teloxide::prelude::{Message, Requester, ResponseResult};
use teloxide::types::ChatId;
use tokio::sync::Mutex;
use crate::admin_command::is_game_active;
use crate::app_state::AppState;

pub async fn handle_message(bot: Bot, msg: Message, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
    let mut state = state.lock().await;

    if !is_game_active(&state, msg.chat.id.0) || msg.text().is_none() {
        println!("stop 1");
        return Ok(())
    } else if msg.text().unwrap().replace(" ", "") != "+" {
        println!("stop 2");
        return Ok(())
    }

    println!("stop 3");

    let Some(user) = &msg.from else { return Ok(()) };

    bot.delete_message(msg.chat.id, msg.id).await?;

    let game = state.active_games.get_mut(&msg.chat.id.0).unwrap();

    if game.players.iter().any(|(id, _)| *id == user.id.0 as i64) {
        bot.send_message(user.id, "You are playing already").await?;
    }

    game.players.insert(user.id.0 as i64, user.username.clone().unwrap());

    let mut players = "".to_string();

    for player in &game.players {
        players.push_str(format!("@{}, ", player.1).as_str());
    }

    players.pop();
    players.pop();

    bot.edit_message_text(msg.chat.id, game.message_id, format!("Game Started:\n\
    Players:\n\
    {players}\n\n\
    Total players: {}", game.players.len()).as_str()).await?;

    Ok(())
}