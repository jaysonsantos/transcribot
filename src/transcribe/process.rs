use color_eyre::{eyre::eyre, Report, Section, SectionExt};
use tokio::{
    io::AsyncReadExt,
    process::{Child, ChildStderr},
};

pub async fn ensure_exit_ok(
    child: &mut Child,
    stderr: &mut ChildStderr,
) -> color_eyre::Result<(), Report> {
    let status = child.wait().await?;

    if status.success() {
        return Ok(());
    }
    let exit_code = format!("{:?}", status.code()).header("status code:");
    let mut buffer = vec![];
    stderr.read_to_end(&mut buffer).await?;
    let stderr = String::from_utf8_lossy(&buffer)
        .to_string()
        .header("stderr:");

    Err(eyre!("Failed to decode audio")
        .section(exit_code)
        .section(stderr))
}
