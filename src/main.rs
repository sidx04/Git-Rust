#[allow(unused_imports)]
use std::env;
use std::fmt::format;
#[allow(unused_imports)]
use std::fs;
use std::io::stdout;
use std::io::BufReader;

use std::ffi::CStr;

use anyhow::Context;
use anyhow::Ok;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::read::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::prelude::*;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

#[derive(Debug)]
enum KnownType {
    Blob,
}

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    /*
    Folder structure:
     - .git/
       - objects/
       - refs/
       - HEAD (should contain "ref: refs/heads/main\n" for a new repository)
    */

    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "Mode not supported!");

            let mut f = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open in .git/objects")?;
            let mut z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);

            let mut buf = Vec::new();

            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;

            let header =
                CStr::from_bytes_until_nul(&buf).expect("\0 NULL is at the end of string only...");

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

            let size = size.parse::<usize>().context(format!(
                ".git/objects file header has invalid size!: {}",
                size
            ))?;

            buf.clear();
            buf.resize(size, 0);
            let slice_buf = z
                .read_exact(&mut buf[..])
                .context("read contents of .git/objects file")?;

            let n = z
                .read(&mut [0])
                .context("validate EOF in .git/object file")?;

            // assert_eq!(n, 0);
            anyhow::ensure!(n == 0, format!(".git/object had {} trailing bytes", n));

            let mut stdout = stdout().lock();
            match known_type {
                KnownType::Blob => stdout
                    .write_all(&buf)
                    .context("Write object contents to buf")?,
            }
        }
    }
    Ok(())
}
