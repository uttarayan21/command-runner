use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: SubCommand,
    #[clap(long, short, default_value = "database.sqlite")]
    pub database: PathBuf,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    #[clap(name = "run")]
    Run(Run),
    #[clap(name = "add")]
    Add(Add),
    #[clap(name = "completions")]
    Completions { shell: clap_complete::Shell },
}

#[derive(Debug, clap::Args)]
pub struct Run {
    #[clap(long, short = 'H', default_value_t = core::net::IpAddr::V4(core::net::Ipv4Addr::LOCALHOST))]
    pub host: core::net::IpAddr,
    #[clap(long, short, default_value_t = 5599u16)]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct Add {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

impl Cli {
    pub fn completions(shell: clap_complete::Shell) {
        let mut command = <Cli as clap::CommandFactory>::command();
        clap_complete::generate(
            shell,
            &mut command,
            env!("CARGO_BIN_NAME"),
            &mut std::io::stdout(),
        );
    }
}
