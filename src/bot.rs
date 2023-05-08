use crate::creator;
use anyhow::Result;
use sscanf::sscanf;
use teloxide_core::adaptors::DefaultParseMode;
use teloxide_core::prelude::*;
use teloxide_core::types::{InputFile, Message, Update, UpdateKind};

pub async fn handle_update(bot: DefaultParseMode<Bot>, update: &Update) -> Result<()> {
    match &update.kind {
        UpdateKind::Message(message) => {
            if handle_commands(bot, update, message).await? {
                return Ok(());
            }
        }
        UpdateKind::EditedMessage(_) => {}
        UpdateKind::ChannelPost(_) => {}
        UpdateKind::EditedChannelPost(_) => {}
        UpdateKind::InlineQuery(_) => {}
        UpdateKind::ChosenInlineResult(_) => {}
        UpdateKind::CallbackQuery(_) => {}
        UpdateKind::ShippingQuery(_) => {}
        UpdateKind::PreCheckoutQuery(_) => {}
        UpdateKind::Poll(_) => {}
        UpdateKind::PollAnswer(_) => {}
        UpdateKind::MyChatMember(_) => {}
        UpdateKind::ChatMember(_) => {}
        UpdateKind::ChatJoinRequest(_) => {}
        UpdateKind::Error(_) => {}
    }

    Ok(())
}

async fn handle_commands(
    bot: DefaultParseMode<Bot>,
    update: &Update,
    message: &Message,
) -> Result<bool> {
    match update.user() {
        Some(user) => {
            log::debug!("New message received from {user:#?} with message {message:#?}.");

            if let Some(text) = message.text() {
                if let Ok(prompt) = sscanf!(text, "/generate {str}") {
                    let chat_id = message.chat.id;

                    log::debug!(
                        "New generation requests received from {:#?}.",
                        update.user()
                    );

                    let edit_message = bot
                        .send_message(chat_id, "⚙️ Processing your request...")
                        .reply_to_message_id(message.id)
                        .await?;

                    match creator::generate(prompt, creator::SEED, true, |text| {
                        let bot = bot.clone();
                        Box::pin(async move {
                            let _ = bot.edit_message_text(chat_id, edit_message.id, text).await;
                        })
                    })
                    .await
                    {
                        Ok(image) => {
                            let _ = bot
                                .send_photo(chat_id, InputFile::memory(image))
                                .reply_to_message_id(message.id)
                                .await?;
                        }
                        Err(e) => {
                            log::error!("Image processing failed: {e:#?}");

                            let _ = bot
                                .send_message(
                                    chat_id,
                                    format!("❌ Photo processing failed: {e:#?}"),
                                )
                                .reply_to_message_id(message.id)
                                .await;
                        }
                    }

                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}
