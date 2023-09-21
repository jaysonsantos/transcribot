use teloxide::{net::Download, prelude::*, types::ChatAction, Bot};
use tokio::io;
use tracing::{info, info_span, instrument, Instrument};

use crate::BUFFER_SIZE;

#[instrument(skip_all, fields(chat_id = %msg.chat.id), err)]
pub async fn deal_with_message(bot: &Bot, msg: &Message) -> color_eyre::Result<()> {
    info!("Processing message");

    let voice = msg.voice().map(|voice| &voice.file.id);
    let audio = msg.audio().map(|audio| &audio.file.id).or(voice);

    let file_id = if let Some(audio) = audio {
        audio
    } else {
        if msg.chat.is_group() {
            return Ok(()); // ignore group messages that are not audio
        }
        info!("No audio file found");
        bot.send_message(msg.chat.id, "Please send me an audio file")
            .reply_to_message_id(msg.id)
            .await?;
        return Ok(());
    };
    bot.send_chat_action(msg.chat.id, ChatAction::Typing)
        .await?;
    let file = bot.get_file(file_id).await?;

    let (reader, writer) = io::duplex(BUFFER_SIZE);

    let lebot = bot.clone();
    let download_file_handle = tokio::spawn(async move {
        info!(?file.path, ?file.id, "downloading file");
        let mut writer = writer;
        lebot.download_file(&file.path, &mut writer).await
    })
    .instrument(info_span!("download_file"));

    let text = crate::transcribe::transcribe_file(reader).await?;
    download_file_handle.await??;
    bot.send_message(msg.chat.id, text)
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
