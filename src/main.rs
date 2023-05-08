use anyhow::Result;
use teloxide_core::prelude::*;
use teloxide_core::types::ParseMode;

use articify_bot::bot;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let bot = Bot::from_env().parse_mode(ParseMode::Html);

    let mut offset = 0;
    loop {
        match bot.get_updates().offset(offset).await {
            Ok(updates) => {
                for update in updates {
                    offset = update.id + 1;

                    match bot::handle_update(bot.clone(), &update).await {
                        Ok(_) => {}
                        Err(err) => log::error!("{:#?}", err),
                    };
                }
            }
            Err(err) => {
                log::error!("{:#?}", err);

                // break;
            }
        }
    }
}
