mod app_state;
mod admin_command;
pub mod types;
mod entity;
mod on_message;

use std::sync::Arc;
use teloxide::{dptree, Bot};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::{Dispatcher, Message};
use teloxide::types::Update;
use tokio::sync::Mutex;
use crate::admin_command::{handle_admin_commands, AdminCommand};
use crate::app_state::AppState;
use crate::on_message::handle_message;

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    let state = Arc::new(Mutex::new(AppState::new().await));

    let handler = Update::filter_message()
        .branch(dptree::entry()
            .filter_command::<AdminCommand>()
            .filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
            .endpoint(handle_admin_commands))
        .branch(dptree::entry()
            .filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
            .endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
