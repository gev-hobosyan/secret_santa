mod app_state;
mod admin_command;
pub mod types;
mod entity;
mod on_message;
pub mod translate;
mod bot_status_change;
mod private_command;

use std::sync::Arc;
use teloxide::{dptree, Bot};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::{Dispatcher, Message};
use teloxide::types::Update;
use tokio::sync::Mutex;
use crate::admin_command::{handle_admin_commands, AdminCommand};
use crate::app_state::AppState;
use crate::bot_status_change::handle_bot_status_change;
use crate::on_message::handle_message;
use crate::private_command::{handle_private_command, PrivateCommand};

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    let state = Arc::new(Mutex::new(AppState::new().await));

    let handler = dptree::entry()
        .branch(Update::filter_my_chat_member().endpoint(handle_bot_status_change))
        .branch(Update::filter_message()
            .branch(dptree::entry()
                .filter_command::<PrivateCommand>()
                .filter(|msg: Message| msg.chat.is_private())
                .endpoint(handle_private_command))
            .branch(dptree::entry()
                .filter_command::<AdminCommand>()
                .filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
                .endpoint(handle_admin_commands))
            .branch(dptree::entry()
                .filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
                .endpoint(handle_message)));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
