use crate::objects::{Kind, Object};
use anyhow::Context;
use anyhow::Ok;

pub(crate) fn invoke(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "Mode not supported!");

    let mut object = Object::read(&object_hash).context("parse object file from hash")?;

    match object.kind {
        Kind::Blob => {
            let mut stdout = std::io::stdout().lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("Write .git/objects file into stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                format!(
                    ".git/object expected size {}, found size {}",
                    object.expected_size, n
                )
            );
        }

        _ => anyhow::bail!("Cannot handle {}", object.kind),
    }

    Ok(())
}
