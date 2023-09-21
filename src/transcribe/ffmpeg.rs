use std::process::Stdio;

use color_eyre::eyre::WrapErr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::Command,
};
use tracing::{info, instrument};

use super::process;
use crate::utils::copy_moved;

#[instrument(skip_all, err)]
pub async fn decode_audio<I, O>(input: I, output: O) -> color_eyre::Result<()>
where
    I: AsyncRead + Unpin + Send + 'static,
    O: AsyncWrite + Unpin + Send,
{
    let mut child = Command::new("ffmpeg")
        .args([
            "-i",
            "-",
            "-f",
            "wav",
            "-c:a",
            "pcm_s16le",
            "-ac", // channels
            "2",
            "-ar", // frequency
            "16000",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .wrap_err("failed to run ffmpeg")?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let input_handle = tokio::spawn(copy_moved(input, stdin));
    let output_handle = copy_moved(stdout, output);
    info!("starting ffmpeg");

    let (input, output) = tokio::join!(input_handle, output_handle);
    let read_bytes = input?.wrap_err("failed to pipe audio into ffmpeg")?;
    let written_bytes = output.wrap_err("failed to read ffmpeg output")?;

    info!(?read_bytes, ?written_bytes, "ffmpeg closed handles");
    process::ensure_exit_ok(&mut child, &mut stderr)
        .await
        .wrap_err("ffmpeg failed")?;
    Ok(())
}
