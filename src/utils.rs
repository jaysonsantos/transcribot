use color_eyre::eyre::WrapErr;
use tokio::{
    io,
    io::{AsyncRead, AsyncWrite},
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn pre_flight() -> color_eyre::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .with(console_subscriber::spawn())
        .with(tracing_error::ErrorLayer::default())
        .init();
    color_eyre::install()?;

    Ok(())
}

pub async fn copy_moved<I, O>(mut input: I, mut output: O) -> color_eyre::Result<u64>
where
    I: AsyncRead + Unpin + Send,
    O: AsyncWrite + Unpin + Send,
{
    let bytes = io::copy(&mut input, &mut output)
        .await
        .wrap_err("failed to copy file")?;
    Ok(bytes)
}
