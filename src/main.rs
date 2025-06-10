use clap::Arg;
use std::env;

use crate::installer::{install, uninstall};

mod build;
mod installer;
mod types;

fn main() {
    let cmd = clap::Command::new("flint")
        .bin_name("flint")
        .styles(CLAP_STYLING)
        .subcommand_required(true)
        .subcommand(clap::command!("install").arg(Arg::new("package")))
        .subcommand(
            clap::command!("remove")
                .arg(Arg::new("package"))
                .alias("uninstall"),
        )
        .subcommand(clap::command!("repair"));

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("install", sub_matches)) => {
            let package = sub_matches
                .get_one::<String>("package")
                .expect("Package is required");
            install(package);
        }
        Some(("remove", sub_matches)) => {
            let package = sub_matches
                .get_one::<String>("package")
                .expect("Package is required");
            uninstall(package);
        }
        Some(("repair", _sub_matches)) => {
            // clean();
        }
        _ => unreachable!(),
    }
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
