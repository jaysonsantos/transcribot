use std::process::Stdio;

use color_eyre::eyre::WrapErr;
use tokio::{
    io,
    io::{AsyncRead, AsyncReadExt},
    process::Command,
};
use tracing::{info, instrument};

use crate::{transcribe::process::ensure_exit_ok, utils::copy_moved, BUFFER_SIZE, WHISPER_MODEL};

#[instrument(skip(wav_file), err, ret)]
pub async fn subprocess_transcribe<I>(wav_file: I) -> color_eyre::Result<String>
where
    I: AsyncRead + Unpin + Send + 'static,
{
    let (mut read_output, write_output) = io::duplex(BUFFER_SIZE);

    let mut child = Command::new("whisper-cpp")
        .args(["-m", WHISPER_MODEL.as_str(), "-l", "auto", "-otxt", "-"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .wrap_err("failed to spawn whisper-cpp")?;
    let mut stderr = child.stderr.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stdin = child.stdin.take().unwrap();

    let input_handle = tokio::spawn(copy_moved(wav_file, stdin));
    let output_handle = copy_moved(stdout, write_output);

    info!("starting whisper-cpp");
    let (input, output) = tokio::join!(input_handle, output_handle);
    input?.wrap_err("failed to pipe wav file to whisper-cpp")?;
    let written = output.wrap_err("failed ot read whisper-cpp result")?;

    ensure_exit_ok(&mut child, &mut stderr)
        .await
        .wrap_err("whisper-cpp did not work")?;
    let mut buffer = Vec::with_capacity(written as usize);

    read_output.read_to_end(&mut buffer).await?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}
