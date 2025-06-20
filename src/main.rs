mod cli;
mod errors;
use errors::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod app;
mod command;
mod database;
mod routes;
#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .pretty(),
        )
        .try_init()
        .change_context(Error)?;

    let args = <cli::Cli as clap::Parser>::parse();
    match args.cmd {
        cli::SubCommand::Run(run) => {
            let database_path = dunce::simplified(&args.database);
            app::App::new(database_path.display().to_string(), run.host, run.port)
                .await?
                .serve()
                .await?;
        }
        cli::SubCommand::Add(add) => {
            let database_path = dunce::simplified(&args.database);
            let database = database::connect(database_path.display().to_string()).await?;
            let command = command::Command::new(add.name, add.command, add.args);
            command.add(&database).await?;
        }
        cli::SubCommand::Completions { shell } => {
            cli::Cli::completions(shell);
        }
    }
    Ok(())
}
