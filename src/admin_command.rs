use std::sync::Arc;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set, TransactionTrait};
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use tokio::sync::{Mutex};
use crate::app_state::{AppState, Lang};
use crate::entity::{game, game_record, player};
use crate::entity::prelude::{Game, GameRecord, Player};
use crate::translate::Translate;
use crate::types::ActiveGame;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule="lowercase")]
pub enum AdminCommand {
    Start,
    Cancel,
    Finish,
    Info,
    Lang(String)
}

pub async fn handle_admin_commands(bot: Bot, msg: Message, cmd: AdminCommand, state: Arc<Mutex<AppState>>) -> ResponseResult<()> {
    let _ = bot.delete_message(msg.chat.id, msg.id).await;

    let Some(user) = &msg.from else { return Ok(()) };

    let t = Translate::init("messages.json").unwrap();
    let _state = state.lock().await;
    let lang = String::from(_state.langs.get(&msg.chat.id.0).unwrap_or(&Lang::EN).clone());
    drop(_state);

    if !is_admin(&bot, &msg).await? {
        bot.send_message(msg.chat.id, t.get(&lang, "admins_only")).await?;
        return Ok(())
    }

    let chat_id = msg.chat.id;

    match cmd {
        AdminCommand::Start => {
            let mut state = state.lock().await;

            if is_game_active(&state, chat_id.0) {
                let game = state.active_games.get(&chat_id.0).unwrap();

                bot.send_message(chat_id, t.with_args(&lang, "already_playing", vec![format!("{}", game.players.len()).as_str(), &game.format_players()])).await?;
                return Ok(())
            }

            if is_game_in_db(&state, chat_id.0).await {
                bot.send_message(chat_id, t.get(&lang, "already_playing")).await?;
                return Ok(())
            };

            let message = bot.send_message(chat_id, "Game started").await?;

            let mut game = ActiveGame::new(message.id);

            game.players.insert(user.id.0 as i64, user.username.clone().unwrap_or("".to_string()));

            bot.edit_message_text(chat_id, message.id, t.with_args(&lang, "registration_open", vec!["1", format!("@{}", user.username.clone().unwrap_or("".to_string())).as_str()])).await?;

            state.active_games.insert(chat_id.0, game);
        }
        AdminCommand::Cancel => {
            let mut state = state.lock().await;

            if !is_game_active(&state, chat_id.0) {
                bot.send_message(chat_id, t.get(&lang, "not_playing")).await?;
                return Ok(())
            }

            let msg_id = state.active_games.get(&chat_id.0).unwrap().message_id;

            bot.delete_message(chat_id, msg_id).await?;

            state.active_games.remove(&chat_id.0);

            bot.send_message(chat_id, t.get(&lang, "canceled")).await?;
        }
        AdminCommand::Finish => {
            let (game, db) = {
                let mut state = state.lock().await;
                let Some(game) = state.active_games.remove(&chat_id.0) else {
                    drop(state);
                    bot.send_message(chat_id, t.get(&lang, "not_playing")).await?;
                    return Ok(())
                };
                (game, state.db.clone())
            };

            if game.players.len() <= 2 {
                let mut state = state.lock().await;
                // let message = bot.send_message(chat_id, t.with_args(&lang, "registration_open", vec![&format!("{}", game.players.len()), &game.format_players()])).await?;
                // let mut game = game.clone();
                // game.message_id = message.id;
                state.active_games.insert(chat_id.0, game);

                bot.send_message(chat_id, t.get(&lang, "not_enough")).await?;

                return Ok(())
            }

            let state = state.lock().await;

            if is_game_in_db(&state, chat_id.0).await {
                bot.send_message(chat_id, t.get(&lang, "already_playing")).await?;
                return Ok(())
            };

            bot.delete_message(chat_id, game.message_id).await?;

            match save_game(&db, chat_id.0, &game).await {
                Ok(pairs) => {
                    bot.send_message(chat_id, t.with_args(&lang, "registration_closed", vec![format!("{}", game.players.len()).as_str(), game.format_players().as_str()])).await?;

                    for (giver, receiver) in pairs {
                        let err = bot.send_message(ChatId(giver.id), t.with_args(&lang, "whom_to_gift", vec![receiver.username.as_str()])).await;

                        println!("{err:?}");
                    }
                }
                Err(err) => {
                    println!("{err:?}");

                    bot.send_message(chat_id, t.get(&lang, "error")).await?;
                }
            }
        }
        AdminCommand::Info => {
            bot.send_message(chat_id, t.get(&lang, "info")).await?;
        }
        AdminCommand::Lang(lang) => {
            let mut state = state.lock().await;
            
            state.langs.insert(chat_id.0, Lang::from(&lang));
        }
    }

    Ok(())
}

async fn is_admin(bot: &Bot, msg: &Message) -> ResponseResult<bool> {
    let Some(user) = &msg.from else { return Ok(false) };

    let admins = bot.get_chat_administrators(msg.chat.id).await?;

    Ok(admins.iter().any(|admin| admin.user.id == user.id))
}

pub fn is_game_active(state: &AppState, chat_id: i64) -> bool {
    state.active_games.iter().any(|(id, _)| id == &chat_id)
}

async fn is_game_in_db(state: &AppState, chat_id: i64) -> bool {
    if let Ok(Some(_)) = Game::find_by_id(chat_id)
        .one(&state.db)
        .await {
        true
    } else { false }
}

async fn save_game(db: &DatabaseConnection, chat_id: i64, game: &ActiveGame) -> Result<Vec<(player::Model, player::Model)>, DbErr> {
    let mut rng: StdRng = rand::make_rng();

    let givers = game.players.clone();

    let pairs: Vec<(i64, i64)> = {
        let mut ids: Vec<i64> = givers.keys().copied().collect();
        ids.shuffle(&mut rng);

        (0..ids.len())
            .map(|i| (ids[i], ids[(i + 1) % ids.len()]))
            .collect()
    };

    let txn = db.begin().await?;

    let givers_clone = givers.clone();

    game::ActiveModel {id: Set(chat_id)}.insert(&txn).await?;

    let players = givers_clone.iter().map(|(id, username)| player::ActiveModel {
        id: Set(*id),
        username: Set(username.clone()),
        wishlist: Default::default(),
    });

    Player::insert_many(players)
        .on_conflict_do_nothing()
        .exec(&txn)
        .await?;

    let records = pairs.clone().into_iter().map(|(giver, receiver)| game_record::ActiveModel {
        id: Default::default(),
        game_id: Set(chat_id),
        player_id: Set(giver),
        receiver_id: Set(receiver),
    });

    GameRecord::insert_many(records)
        .exec(&txn)
        .await?;

    let players: Vec<(player::Model, player::Model)> = pairs.iter().map(|(giver, receiver)| {
        (player::Model {
            id: *giver,
            username: givers.get(giver).unwrap().clone(),
            wishlist: "".to_string(),
        }, player::Model {
            id: *giver,
            username: givers.get(receiver).unwrap().clone(),
            wishlist: "".to_string(),
        })
    }).collect();

    txn.commit().await?;

    Ok(players)
}