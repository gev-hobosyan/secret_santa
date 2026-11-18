use std::sync::Arc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use crate::app_state::{AppState, Lang};
use crate::entity::{game_record, player};
use crate::entity::prelude::{GameRecord, Player};
use crate::translate::Translate;

#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase")]
pub enum PrivateCommand {
    Wishlist(String),
    Lang(String)
}

pub async fn handle_private_command(bot: Bot, msg: Message, cmd: PrivateCommand, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
    let _ = bot.delete_message(msg.chat.id, msg.id).await;

    let Some(user) = &msg.from else { return Ok(()) };

    let t = Translate::init("messages.json").unwrap();
    let _state = state.lock().await;
    let lang = String::from(_state.langs.get(&msg.chat.id.0).unwrap_or(&Lang::EN).clone());
    drop(_state);

    let chat_id = msg.chat.id;

    match cmd {
        PrivateCommand::Wishlist(msg) => {
            let state = state.lock().await;

            let res = Player::find_by_id(user.id.0 as i64).one(&state.db).await;

            match res {
                Ok(player) => {
                    if let None = player.clone() {
                        bot.send_message(user.id, t.get(&lang, "not_playing")).await?;

                        return Ok(())
                    }

                    let player = player.unwrap();

                    let mut active: player::ActiveModel = player.into();

                    active.wishlist = Set(msg.clone());

                    match active.update(&state.db).await {
                        Ok(player) => {
                            let res = GameRecord::find()
                                .filter(game_record::Column::ReceiverId.eq(player.id))
                                .all(&state.db)
                                .await;

                            if let Ok(records) = res {
                                for record in records {
                                    bot.send_message(ChatId(record.player_id), t.with_args(&lang, "wishlist_updated", vec![&player.username, &msg])).await?;
                                }
                            }
                        }
                        Err(err) => {
                            println!("{err:?}");

                            bot.send_message(user.id, t.get(&lang, "error")).await?;
                        }
                    }
                }
                Err(err) => {
                    println!("{err:?}");

                    bot.send_message(user.id, t.get(&lang, "error")).await?;
                }
            }
        }
        PrivateCommand::Lang(lang) => {
            let mut state = state.lock().await;

            state.langs.insert(chat_id.0, Lang::from(&lang));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use teloxide::Bot;
    use teloxide::prelude::{ChatId, Message, UserId};
    use teloxide::types::{Chat, ChatKind, ChatPrivate, MessageKind, User};
    use tokio::sync::Mutex;
    use crate::app_state::AppState;
    use crate::private_command::{handle_private_command, PrivateCommand};

    #[tokio::test]
    async fn test_wishlist() {
        let bot = Bot::from_env();
        let msg = Message {
            id: Default::default(),
            thread_id: None,
            from: Some(User {
                // Replace with actual ID
                id: UserId(0000000000),
                is_bot: false,
                first_name: "".to_string(),
                last_name: None,
                username: Some("".to_string()),
                language_code: None,
                is_premium: false,
                added_to_attachment_menu: false,
            }),
            sender_chat: None,
            date: Default::default(),
            // Replace with actual ID
            chat: Chat { id: ChatId(0000000000), kind: ChatKind::Private(ChatPrivate {
                username: Some("".to_string()),
                first_name: None,
                last_name: None,
            }) },
            is_topic_message: false,
            via_bot: None,
            sender_business_bot: None,
            kind: MessageKind::Empty {},
        };

        let state = Arc::new(Mutex::new(AppState::new().await));

        handle_private_command(bot, msg, PrivateCommand::Wishlist("test wishlist\nyahoo\n\n".to_string()), state).await.unwrap();
    }
}