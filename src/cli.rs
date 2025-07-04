use std::path::PathBuf;

use crate::command::Identifier;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: SubCommand,
    #[clap(
        long,
        short,
        default_value = "/run/command-runner/database.sqlite",
        global = true
    )]
    pub database: Option<PathBuf>,
    #[clap(
        long,
        short,
        default_value = "/etc/command-runner/config.toml",
        global = true
    )]
    pub config: PathBuf,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    #[clap(name = "run")]
    Run(Run),
    #[clap(name = "add")]
    Add(Add),
    #[clap(name = "list")]
    List(List),
    #[clap(name = "rm", alias = "delete")]
    Rm(Rm),
    #[clap(name = "completions")]
    Completions { shell: clap_complete::Shell },
}

#[derive(Debug, clap::Args)]
pub struct Run {
    #[clap(long, short = 'H')]
    pub host: Option<core::net::IpAddr>,
    #[clap(long, short)]
    pub port: Option<u16>,
}

#[derive(Debug, clap::Args)]
#[clap(group = clap::ArgGroup::new("like").required(true))]
pub struct Rm {
    #[clap(long, short = 'n', group = "like")]
    pub name: Option<String>,
    #[clap(long, short = 'C', group = "like")]
    pub command: Option<String>,
    #[clap(long, short = 'i', help = "Remove by ID", group = "like")]
    pub id: Option<uuid::Uuid>,
    #[clap(
        long,
        short = 'a',
        help = "Remove all commands",
        default_value_t = false,
        group = "like"
    )]
    pub all: bool,
}
impl Rm {
    pub fn to_identifier(&self) -> crate::Result<Identifier> {
        Ok(if let Some(id) = self.id {
            Identifier::Id(id)
        } else if let Some(name) = &self.name {
            Identifier::Name(name.clone())
        } else if let Some(command) = &self.command {
            Identifier::Like(command.clone())
        } else {
            Err(crate::errors::Error::new().attach_printable("No identifier provided for removal"))?
        })
    }
}

#[derive(Debug, clap::Args)]
pub struct Add {
    #[clap(
        long,
        short,
        help = "Ignore existing commands with the same name",
        default_value_t = false,
        group = "add_mode"
    )]
    pub ignore: bool,
    #[clap(
        long,
        short,
        help = "Replace existing commands with the same name",
        default_value_t = false,
        group = "add_mode"
    )]
    pub replace: bool,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, clap::Args)]
pub struct List {
    #[clap(long, short = 'n', group = "like")]
    pub name: Option<String>,
    #[clap(long, short = 'C', group = "like")]
    pub command: Option<String>,
    #[clap(long, short = 'v', help = "Enable verbose output")]
    pub verbose: bool,
}

impl Cli {
    pub fn completions(shell: clap_complete::Shell) {
        let mut command = Self::command();
        clap_complete::generate(
            shell,
            &mut command,
            env!("CARGO_BIN_NAME"),
            &mut std::io::stdout(),
        );
    }

    pub fn command() -> clap::Command {
        <Cli as clap::CommandFactory>::command()
    }

    #[cfg(test)]
    pub fn verify() {
        Self::command().debug_assert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        Cli::verify();
    }
}
