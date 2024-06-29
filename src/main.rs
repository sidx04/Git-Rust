#[allow(unused_imports)]
use std::env;
use std::io::BufReader;
use std::{fs, path::Path};

use std::ffi::CStr;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::write::ZlibEncoder;
use flate2::{write::ZlibDecoder, Compression};
use sha1::{Digest, Sha1};
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
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

#[derive(Debug)]
enum KnownType {
    Blob,
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
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
        }

        Command::HashObject { write, file } => {
            fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stat =
                    std::fs::metadata(&file).with_context(|| format!("stat {}", file.display()))?;
                let writer = ZlibEncoder::new(writer, Compression::default());
                let mut writer = HashWriter {
                    writer,
                    hasher: Sha1::new(),
                };
                write!(writer, "blob ")?;
                write!(writer, "{}\0", stat.len())?;
                let mut file = std::fs::File::open(&file)
                    .with_context(|| format!("open {}", file.display()))?;
                std::io::copy(&mut file, &mut writer).context("stream file into blob")?;
                let _ = writer.writer.finish()?;
                let hash = writer.hasher.finalize();
                Ok(hex::encode(hash))
            }

            let hash = if write {
                let tmp = "temporary";
                let hash = write_blob(
                    &file,
                    std::fs::File::create(tmp).context("construct temporary file for blob")?,
                )
                .context("write out blob object")?;

                fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
                    .context("create subdir of .git/objects")?;

                std::fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("move blob file into .git/objects")?;
                hash
            } else {
                write_blob(&file, std::io::sink()).context("write out blob object")?
            };

            println!("{hash}");
        }
    }
    Ok(())
}
