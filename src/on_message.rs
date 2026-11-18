use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::{Message, Requester, ResponseResult};
use tokio::sync::Mutex;
use crate::admin_command::is_game_active;
use crate::app_state::{AppState, Lang};
use crate::translate::Translate;

pub async fn handle_message(bot: Bot, msg: Message, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
    let _state = state.lock().await;
    let lang = String::from(_state.langs.get(&msg.chat.id.0).unwrap_or(&Lang::EN).clone());
    drop(_state);

    let t = Translate::init("messages.json").unwrap();

    let mut state = state.lock().await;

    if !is_game_active(&state, msg.chat.id.0) || msg.text().is_none() {
        return Ok(())
    } else if msg.text().unwrap().replace(" ", "") != "+" {
        return Ok(())
    }

    let Some(user) = &msg.from else { return Ok(()) };

    let _ = bot.delete_message(msg.chat.id, msg.id).await;

    let game = state.active_games.get_mut(&msg.chat.id.0).unwrap();

    if game.players.iter().any(|(id, _)| *id == user.id.0 as i64) {
        bot.send_message(user.id, t.get(&lang, "private_playing")).await?;
        return Ok(())
    }

    game.players.insert(user.id.0 as i64, user.username.clone().unwrap_or(user.first_name.clone()));

    bot.edit_message_text(msg.chat.id, game.message_id, t.with_args(&lang, "registration_open", vec![&format!("{}", game.players.len()), &game.format_players()])).await?;

    Ok(())
}