use teloxide::Bot;
use teloxide::prelude::{Requester, ResponseResult};
use teloxide::types::ChatMemberUpdated;
use crate::translate::Translate;

pub async fn handle_bot_status_change(bot: Bot, upd: ChatMemberUpdated) -> ResponseResult<()> {
    let t = Translate::init("messages.json").unwrap();
    let was_in_group = upd.old_chat_member.is_present();
    let is_in_group = upd.new_chat_member.is_present();
    let is_group = upd.chat.is_group() || upd.chat.is_supergroup();

    if is_group && !was_in_group && is_in_group {
        bot.send_message(upd.chat.id, t.get("en", "info")).await?;
    } else if is_group && was_in_group && !is_in_group {
        // TODO: implement cleanup
        println!("Deleted from a group");
    }

    Ok(())
}