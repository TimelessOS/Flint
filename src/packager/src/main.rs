#![warn(clippy::pedantic)]

use anyhow::Result;
use clap::Parser;

use crate::packager::RepoCommands;

mod packager;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmds: RepoCommands,
}

fn main() -> Result<()> {
    let args = Args::parse();

    packager::main(args.cmds)
}
