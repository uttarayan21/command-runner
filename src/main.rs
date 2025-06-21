mod cli;
mod errors;
use errors::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod app;
mod command;
mod config;
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
        cli::SubCommand::Run(_) => {
            let config = config::Config::try_new(&args)?;
            let database_path = dunce::simplified(&config.database);
            app::App::new(
                database_path.display().to_string(),
                config.host,
                config.port,
            )
            .await?
            .serve()
            .await?;
        }
        cli::SubCommand::Add(ref add) => {
            let config = config::Config::try_new(&args)?;
            let database_path = dunce::simplified(&config.database);
            let database = database::connect(database_path.display().to_string()).await?;
            let command =
                command::Command::new(add.name.clone(), add.command.clone(), add.args.clone());
            let mode = match (add.ignore, add.replace) {
                (true, false) => command::CommandAddMode::Ignore,
                (false, true) => command::CommandAddMode::Replace,
                _ => command::CommandAddMode::Error,
            };
            command.add(&database, mode).await?;
        }
        cli::SubCommand::List(ref list) => {
            let config = config::Config::try_new(&args)?;
            let database_path = dunce::simplified(&config.database);
            let database = database::connect(database_path.display().to_string()).await?;
            let cmds = if let Some(like) = list.name.clone().or(list.command.clone()) {
                command::Command::like(&database, like).await?
            } else {
                command::Command::list(&database).await?
            };
            cmds.iter().for_each(|cmd| {
                if list.verbose {
                    print!("{}: ", cmd.id);
                }
                println!("{}: {} {}", cmd.name, cmd.command, cmd.args.join(" "));
            });
        }
        cli::SubCommand::Rm(ref rm) => {
            let config = config::Config::try_new(&args)?;
            let database_path = dunce::simplified(&config.database);
            let database = database::connect(database_path.display().to_string()).await?;
            if rm.all {
                command::Command::delete_all(&database).await?;
            } else {
                let identifier = rm.to_identifier()?;
                let command = command::Command::identifier(&database, identifier).await?;
                command.delete(&database).await?;
            }
        }
        cli::SubCommand::Completions { shell } => {
            cli::Cli::completions(shell);
        }
    }
    Ok(())
}
