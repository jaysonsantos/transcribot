mod ffmpeg;
mod process;
mod whisper;

use tokio::{io, io::AsyncRead};
use tracing::{info_span, instrument, Instrument};

use crate::BUFFER_SIZE;

#[instrument(skip_all, err)]
pub async fn transcribe_file<I>(maybe_encoded_file: I) -> color_eyre::Result<String>
where
    I: AsyncRead + Unpin + Send + 'static,
{
    let (read_wav, write_wav) = io::duplex(BUFFER_SIZE);

    let decode_handle = tokio::spawn(ffmpeg::decode_audio(maybe_encoded_file, write_wav))
        .instrument(info_span!("decode_audio"));
    let transcribed = whisper::subprocess_transcribe(read_wav).await?;
    decode_handle.await??;

    Ok(transcribed)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use camino::Utf8PathBuf;
    use color_eyre::{eyre::WrapErr, Result};
    use tokio::{fs::File, io, spawn, time::sleep};
    use tracing::{info, info_span, Instrument};

    use crate::{
        transcribe::transcribe_file,
        utils::{copy_moved, pre_flight},
        BUFFER_SIZE,
    };

    #[tokio::test]
    async fn cursor() -> Result<()> {
        pre_flight()?;
        let path = Utf8PathBuf::from(".")
            .canonicalize_utf8()?
            .join("fixtures/hello-world.ogg");
        let f = File::open(&path).await.wrap_err(path)?;
        let (reader, writer) = io::duplex(BUFFER_SIZE);

        let transcribe_handle =
            spawn(transcribe_file(reader)).instrument(info_span!("transcribe_file"));

        info!("copying file");
        let copied_bytes = copy_moved(f, writer).await?;
        info!(?copied_bytes, "copied file");

        let text = transcribe_handle
            .await?
            .wrap_err("failed to transcribe file")?;

        sleep(Duration::from_secs(2)).await;
        assert_eq!(text, "\n[00:00:00.000 --> 00:00:02.000]   Hello world\n\n");
        Ok(())
    }
}
