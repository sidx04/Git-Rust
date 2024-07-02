use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

use anyhow::{Context, Ok};

use crate::objects::{Kind, Object};

pub(crate) fn invoke(name_only: bool, tree_hash: &String) -> anyhow::Result<()> {
    let mut object = Object::read(tree_hash).context("parse tree object file")?;

    match object.kind {
        Kind::Tree => {
            let mut buf = Vec::new();
            let mut hash_buf = [0; 20];
            let mut stdout = std::io::stdout();
            loop {
                buf.clear();
                let n = object
                    .reader
                    .read_until(0, &mut buf)
                    .context("read mode and name, which will terminate at a `\0'")?;
                if n == 0 {
                    break;
                }

                object
                    .reader
                    .read_exact(&mut hash_buf[..])
                    .context("read tree entry object hash")?;

                let mode_name = CStr::from_bytes_until_nul(&buf).context("invalid tree entry")?;
                let mut bits = mode_name.to_bytes().splitn(2, |&b| b == b' ');

                let mode = bits.next().expect("split always yiels once");
                let name = bits
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("tree entry has no file name!"))?;

                if name_only {
                    stdout
                        .write_all(&name)
                        .context("write tree entry name to stdout")?;
                } else {
                    let mode = std::str::from_utf8(mode).context("mode is always utf-8")?;
                    let hash = hex::encode(&hash_buf);
                    let object = Object::read(&hash)
                        .with_context(|| format!("read object for tree entry hash: {}", hash))?;

                    write!(stdout, "{:0>6} {} {}\t", mode, object.kind, hash)
                        .context("write tree entry hash to stdout")?;

                    stdout
                        .write_all(&name)
                        .context("write tree entry name to stdout")?;
                }
                writeln!(stdout, "")?;
            }
        }
        _ => anyhow::bail!("Cannot handle {}", object.kind),
    }
    Ok(())
}
