use anyhow::Context;
use anyhow::Ok;
use core::fmt;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn read(hash: &String) -> anyhow::Result<Object<impl BufRead>> {
        let f = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);

        let mut buf = Vec::new();

        z.read_until(0, &mut buf)
            .context("read header from .git/objects")?;

        let header =
            CStr::from_bytes_until_nul(&buf).expect("\0 NULL is at the end of string only...");

        let header = header
            .to_str()
            .context(".git/objects file header is not in UTF-8")?;

        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(format!(
                ".git/objects file header did not start with a known type: {}",
                header
            ));
        };

        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!(format!("cannot process for type: {}", kind)),
        };

        let size = size.parse::<u64>().context(format!(
            ".git/objects file header has invalid size!: {}",
            size
        ))?;

        // if decompressed file is too long, this won't throw an error
        // but not vulnerable to a zipbomb either

        let z = z.take(size);

        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}
