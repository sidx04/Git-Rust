#[allow(unused_imports)]
use std::env;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

pub(crate) mod commands;

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

fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    let args = Args::parse();

    match args.command {
        Command::Init => commands::init::invoke().context("initialise `/.git` directory")?,

        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, object_hash).context("implement cat-file")?,

        Command::HashObject { write, file } => {
            commands::hash_object::invoke(write, &file).context("implement hash-object")?;
        }
    }
    Ok(())
}
