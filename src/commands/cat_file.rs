use anyhow::Context;
use anyhow::Ok;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug)]
enum KnownType {
    Blob,
}

pub(crate) fn invoke(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "Mode not supported!");

    let f = std::fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;
    let z = ZlibDecoder::new(f);
    let mut z = BufReader::new(z);

    let mut buf = Vec::new();

    z.read_until(0, &mut buf)
        .context("read header from .git/objects")?;

    let header = CStr::from_bytes_until_nul(&buf).expect("\0 NULL is at the end of string only...");

    let header = header
        .to_str()
        .context(".git/objects file header is not in UTF-8")?;

    let Some((known_type, size)) = header.split_once(' ') else {
        anyhow::bail!(format!(
            ".git/objects file header did not start with a known type: {}",
            header
        ));
    };

    let known_type = match known_type {
        "blob" => KnownType::Blob,
        _ => anyhow::bail!(format!("Cannot process for type: {}", known_type)),
    };

    let size = size.parse::<u64>().context(format!(
        ".git/objects file header has invalid size!: {}",
        size
    ))?;

    // if decompressed file is too long, this won't throw an error
    // but not vulnerable to a zipbomb either

    let mut z = z.take(size);

    match known_type {
        KnownType::Blob => {
            let mut stdout = std::io::stdout().lock();
            let n = std::io::copy(&mut z, &mut stdout)
                .context("Write .git/objects file into stdout")?;
            anyhow::ensure!(
                n == size,
                format!(".git/object expected size {}, found size {}", size, n)
            );
        }
    }

    Ok(())
}
