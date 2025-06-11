use anyhow::Result;
use clap::{Parser, Subcommand};
#[cfg(feature = "packager")]
use flint::RepoCommands;

use crate::installer::{install, repair, uninstall};

mod installer;

#[derive(Parser)]
#[clap(version)]
#[clap(styles = CLAP_STYLING)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Remove a package
    Remove {
        /// The name of the package to remove
        package: String,
    },

    /// Install a package
    Install {
        /// The name of the package to install
        package: String,

        /// The repo to install the package from
        ///
        /// This can either be a URL or a path.
        #[clap(short, long)]
        repo: Option<String>,
    },

    /// Repair broken packages
    Repair {},
    /// Interact with a Repo
    #[cfg(feature = "packager")]
    Repo {
        #[command(subcommand)]
        cmds: RepoCommands,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Commands::Remove { package } => uninstall(&package),
        Commands::Install { package, repo } => install(&package, repo),
        Commands::Repair {} => repair(),
        #[cfg(feature = "packager")]
        Commands::Repo { cmds } => flint_packager::main(cmds),
    }
}

const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
