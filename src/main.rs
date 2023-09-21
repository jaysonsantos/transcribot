use std::env::var;

use camino::Utf8PathBuf;
use color_eyre::Result;
use lazy_static::lazy_static;
use teloxide::prelude::*;
use tracing::{error, info};

mod telegram;
mod transcribe;
mod utils;

const BUFFER_SIZE: usize = 65_536;

lazy_static! {
    static ref WHISPER_PATH: Utf8PathBuf = var("WHISPER_PATH")
        .map(Utf8PathBuf::from)
        .expect("WHISPER_PATH is not set");
    static ref WHISPER_MODEL: Utf8PathBuf = {
        let model = var("WHISPER_MODEL").expect("WHISPER_MODEL is not set");
        WHISPER_PATH.join("models").join(model)
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::pre_flight()?;

    let _ = *WHISPER_PATH;
    assert!(WHISPER_MODEL.exists());

    let bot = Bot::from_env();
    let me = bot.get_me().await?;
    info!(?me, "Starting with");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Err(err) = telegram::deal_with_message(&bot, &msg).await {
            error!(?err, "Something went wrong");
            bot.send_message(msg.chat.id, format!("Something went wrong: \n{err:?}"))
                .await?;
        }
        Ok(())
    })
    .await;
    Ok(())
}
