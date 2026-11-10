mod app_state;
mod admin_command;
pub mod types;

use std::sync::Arc;
use teloxide::{dptree, Bot};
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::{Dispatcher, Message};
use teloxide::types::Update;
use tokio::sync::Mutex;
use crate::admin_command::handle_admin_commands;
use crate::app_state::AppState;

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    let state = Arc::new(Mutex::new(AppState::new()));

    let handler = Update::filter_message()
        .branch(dptree::entry().filter(|msg: Message| msg.chat.is_group() || msg.chat.is_supergroup())
            .endpoint(handle_admin_commands));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
